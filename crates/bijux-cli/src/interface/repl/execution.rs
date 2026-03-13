use crate::contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use crate::interface::cli::dispatch::run_app;
use crate::routing::parser::root_command;

use super::history::push_history;
use super::types::{
    ReplError, ReplEvent, ReplFrame, ReplInput, ReplSession, ReplStream, META_PREFIX,
    REPL_LAST_ERROR_MAX_CHARS, REPL_MULTILINE_BUFFER_MAX_CHARS,
};

fn parse_shell_tokens_lossy(input: &str) -> Vec<String> {
    shlex::split(input)
        .unwrap_or_else(|| input.split_whitespace().map(ToString::to_string).collect())
}

fn bounded_error_message(message: &str) -> String {
    message.chars().filter(|ch| !ch.is_control()).take(REPL_LAST_ERROR_MAX_CHARS).collect()
}

fn set_last_error(session: &mut ReplSession, message: &str) {
    session.last_error = Some(bounded_error_message(message));
}

fn parse_shell_tokens_strict(input: &str) -> Result<Vec<String>, ReplError> {
    shlex::split(input).ok_or_else(|| {
        let snippet = input.chars().filter(|ch| !ch.is_control()).take(256).collect::<String>();
        ReplError::InvalidCommandInput(format!("shell tokenization failed: {snippet}"))
    })
}

fn output_format_from_name(name: &str) -> Option<OutputFormat> {
    match name {
        "json" => Some(OutputFormat::Json),
        "yaml" => Some(OutputFormat::Yaml),
        "text" => Some(OutputFormat::Text),
        _ => None,
    }
}

fn output_format_name(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Json => "json",
        OutputFormat::Yaml => "yaml",
        OutputFormat::Text => "text",
    }
}

/// Build argv using the same tokenization path REPL execution uses.
#[must_use]
pub fn repl_argv_from_line(line: &str) -> Vec<String> {
    let tokenized = parse_shell_tokens_lossy(line);
    std::iter::once("bijux".to_string()).chain(tokenized).collect()
}

fn needs_multiline_continuation(line: &str) -> bool {
    let trimmed = line.trim_end();
    let trailing_backslashes = trimmed.chars().rev().take_while(|ch| *ch == '\\').count();
    trailing_backslashes % 2 == 1
}

fn strip_single_continuation_backslash(line: &str) -> &str {
    line.strip_suffix('\\').unwrap_or(line)
}

fn render_meta_help(path: &[String]) -> Result<String, ReplError> {
    let mut command = root_command();
    let mut curr = &mut command;
    for segment in path {
        if let Some(next) = curr.find_subcommand_mut(segment) {
            curr = next;
        } else {
            return Err(ReplError::InvalidMetaCommand(format!(
                "unknown help topic: {}",
                path.join(" ")
            )));
        }
    }

    let mut bytes = Vec::new();
    if curr.write_long_help(&mut bytes).is_ok() {
        Ok(String::from_utf8(bytes).unwrap_or_else(|_| "Unable to render help\n".to_string()))
    } else {
        Ok("Unable to render help\n".to_string())
    }
}

fn handle_meta_command(session: &mut ReplSession, line: &str) -> Result<ReplEvent, ReplError> {
    let raw = line.trim_start_matches(META_PREFIX).trim();
    let tokens = parse_shell_tokens_strict(raw)?;
    if tokens.is_empty() {
        return Err(ReplError::InvalidMetaCommand(line.to_string()));
    }

    match tokens[0].as_str() {
        "help" => {
            let body = render_meta_help(&tokens[1..])?;
            Ok(ReplEvent::Continue(Some(ReplFrame {
                stream: ReplStream::Stdout,
                content: if body.ends_with('\n') { body } else { format!("{body}\n") },
            })))
        }
        "set" if tokens.len() == 3 => {
            match (tokens[1].as_str(), tokens[2].as_str()) {
                ("trace", "on") => session.trace_mode = true,
                ("trace", "off") => session.trace_mode = false,
                ("quiet", "on") => session.policy.quiet = true,
                ("quiet", "off") => session.policy.quiet = false,
                ("format", value) => {
                    session.policy.output_format = output_format_from_name(value)
                        .ok_or_else(|| ReplError::InvalidMetaCommand(line.to_string()))?;
                }
                _ => return Err(ReplError::InvalidMetaCommand(line.to_string())),
            }

            Ok(ReplEvent::Continue(Some(ReplFrame {
                stream: ReplStream::Stdout,
                content: "ok\n".to_string(),
            })))
        }
        "exit" | "quit" if tokens.len() == 1 => Ok(ReplEvent::Exit(None)),
        _ => Err(ReplError::InvalidMetaCommand(line.to_string())),
    }
}

fn apply_session_policy_to_argv(session: &ReplSession, line_argv: &[String]) -> Vec<String> {
    let mut argv = vec!["bijux".to_string()];

    argv.push("--format".to_string());
    argv.push(output_format_name(session.policy.output_format).to_string());

    argv.push(
        match session.policy.pretty_mode {
            PrettyMode::Pretty => "--pretty",
            PrettyMode::Compact => "--no-pretty",
        }
        .to_string(),
    );

    if session.policy.quiet {
        argv.push("--quiet".to_string());
    }

    argv.push("--color".to_string());
    argv.push(
        match session.policy.color_mode {
            ColorMode::Auto => "auto",
            ColorMode::Always => "always",
            ColorMode::Never => "never",
        }
        .to_string(),
    );

    argv.push("--log-level".to_string());
    argv.push(if session.trace_mode {
        "trace".to_string()
    } else {
        match session.policy.log_level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            _ => "info",
        }
        .to_string()
    });

    if line_argv.len() > 1 {
        argv.extend_from_slice(&line_argv[1..]);
    }
    argv
}

/// Execute one REPL input event with interrupt/EOF-safe behavior.
pub fn execute_repl_input(
    session: &mut ReplSession,
    input: ReplInput,
) -> Result<ReplEvent, ReplError> {
    match input {
        ReplInput::Interrupt => {
            session.pending_multiline = None;
            session.commands_executed += 1;
            session.last_exit_code = 130;
            set_last_error(session, "Interrupted");
            Ok(ReplEvent::Interrupted(ReplFrame {
                stream: ReplStream::Stderr,
                content: "Interrupted\n".to_string(),
            }))
        }
        ReplInput::Eof => {
            if session.pending_multiline.take().is_some() {
                session.commands_executed += 1;
                session.last_exit_code = 2;
                set_last_error(session, "EOF received with pending multiline command");
            }
            Ok(ReplEvent::Exit(None))
        }
        ReplInput::Line(line) => {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return Ok(ReplEvent::Continue(None));
            }

            if needs_multiline_continuation(trimmed) {
                let chunk = strip_single_continuation_backslash(trimmed).trim_end();
                let pending = match session.pending_multiline.take() {
                    Some(existing) => format!("{existing}\n{chunk}"),
                    None => chunk.to_string(),
                };
                if pending.chars().count() > REPL_MULTILINE_BUFFER_MAX_CHARS {
                    session.commands_executed += 1;
                    session.last_exit_code = 2;
                    set_last_error(
                        session,
                        &format!(
                            "multiline command exceeded {} characters",
                            REPL_MULTILINE_BUFFER_MAX_CHARS
                        ),
                    );
                    return Err(ReplError::InvalidCommandInput(
                        "multiline command buffer limit exceeded".to_string(),
                    ));
                }
                session.pending_multiline = Some(pending);
                return Ok(ReplEvent::Continue(None));
            }

            let final_line = if let Some(existing) = session.pending_multiline.take() {
                format!("{existing}\n{trimmed}")
            } else {
                trimmed.to_string()
            };

            if final_line.starts_with(META_PREFIX) {
                let outcome = handle_meta_command(session, &final_line);
                match &outcome {
                    Ok(ReplEvent::Continue(_)) | Ok(ReplEvent::Exit(_)) => {
                        session.commands_executed += 1;
                        session.last_exit_code = 0;
                        session.last_error = None;
                    }
                    Ok(ReplEvent::Interrupted(_)) => {
                        session.commands_executed += 1;
                        session.last_exit_code = 130;
                        set_last_error(session, "Interrupted");
                    }
                    Err(error) => {
                        session.commands_executed += 1;
                        session.last_exit_code = 2;
                        set_last_error(session, &error.to_string());
                    }
                }
                return outcome;
            }

            let tokenized = match parse_shell_tokens_strict(&final_line) {
                Ok(value) => value,
                Err(error) => {
                    session.commands_executed += 1;
                    session.last_exit_code = 2;
                    set_last_error(session, &error.to_string());
                    return Err(error);
                }
            };
            let argv = std::iter::once("bijux".to_string()).chain(tokenized).collect::<Vec<_>>();
            let history_line = final_line.replace('\n', " ");
            push_history(session, &history_line);

            let effective_argv = apply_session_policy_to_argv(session, &argv);
            let result = match run_app(&effective_argv) {
                Ok(value) => value,
                Err(error) => {
                    session.commands_executed += 1;
                    session.last_exit_code = 1;
                    set_last_error(session, &error.to_string());
                    return Err(ReplError::Core(error.to_string()));
                }
            };

            session.commands_executed += 1;
            session.last_exit_code = result.exit_code;

            let frame = if result.exit_code != 0 && !result.stderr.is_empty() {
                set_last_error(session, &result.stderr);
                Some(ReplFrame { stream: ReplStream::Stderr, content: result.stderr })
            } else if !result.stdout.is_empty() {
                if result.exit_code == 0 {
                    session.last_error = None;
                }
                Some(ReplFrame { stream: ReplStream::Stdout, content: result.stdout })
            } else if !result.stderr.is_empty() {
                if result.exit_code == 0 {
                    session.last_error = None;
                } else {
                    set_last_error(session, &result.stderr);
                }
                Some(ReplFrame { stream: ReplStream::Stderr, content: result.stderr })
            } else {
                if result.exit_code == 0 {
                    session.last_error = None;
                } else {
                    set_last_error(
                        session,
                        &format!("command failed with exit code {}", result.exit_code),
                    );
                }
                None
            };

            Ok(ReplEvent::Continue(frame))
        }
    }
}

/// Backward-compatible one-line execution adapter.
pub fn execute_repl_line(
    session: &mut ReplSession,
    line: &str,
) -> Result<Option<ReplFrame>, ReplError> {
    match execute_repl_input(session, ReplInput::Line(line.to_string()))? {
        ReplEvent::Continue(frame) => Ok(frame),
        ReplEvent::Exit(frame) => Ok(frame),
        ReplEvent::Interrupted(frame) => Ok(Some(frame)),
    }
}

#[cfg(test)]
mod tests {
    use super::{execute_repl_input, execute_repl_line, needs_multiline_continuation};
    use crate::interface::repl::session::startup_repl;
    use crate::interface::repl::types::{ReplError, ReplInput, REPL_MULTILINE_BUFFER_MAX_CHARS};

    #[test]
    fn malformed_shell_input_returns_deterministic_invalid_input_error() {
        let (mut session, _) = startup_repl("", None);
        let result = execute_repl_line(&mut session, "status --config-path \"unterminated");
        assert!(matches!(result, Err(ReplError::InvalidCommandInput(_))));
        assert_eq!(session.last_exit_code, 2);
        assert_eq!(session.commands_executed, 1);
        assert!(session.last_error.is_some());
    }

    #[test]
    fn meta_set_requires_exact_arity_and_sets_usage_exit_code() {
        let (mut session, _) = startup_repl("", None);
        let result = execute_repl_line(&mut session, ":set format json extra");
        assert!(matches!(result, Err(ReplError::InvalidMetaCommand(_))));
        assert_eq!(session.last_exit_code, 2);
        assert!(session.last_error.as_deref().unwrap_or_default().contains("invalid repl command"));
    }

    #[test]
    fn successful_command_clears_previous_error_state() {
        let (mut session, _) = startup_repl("", None);

        let _ = execute_repl_line(&mut session, "config get");
        assert!(session.last_error.is_some());

        let result = execute_repl_line(&mut session, "status --format json --no-pretty")
            .expect("status command should execute");
        assert!(result.is_some());
        assert_eq!(session.last_exit_code, 0);
        assert!(session.last_error.is_none());
    }

    #[test]
    fn meta_help_unknown_topic_is_usage_error() {
        let (mut session, _) = startup_repl("", None);
        let result = execute_repl_line(&mut session, ":help definitely-missing-command");
        assert!(matches!(result, Err(ReplError::InvalidMetaCommand(_))));
        assert_eq!(session.last_exit_code, 2);
        assert_eq!(session.commands_executed, 1);
    }

    #[test]
    fn interrupt_updates_last_error_and_counter() {
        let (mut session, _) = startup_repl("", None);
        let event = execute_repl_input(&mut session, ReplInput::Interrupt)
            .expect("interrupt should return event");
        assert!(matches!(event, crate::interface::repl::types::ReplEvent::Interrupted(_)));
        assert_eq!(session.last_exit_code, 130);
        assert_eq!(session.commands_executed, 1);
        assert_eq!(session.last_error.as_deref(), Some("Interrupted"));
    }

    #[test]
    fn continuation_requires_odd_trailing_backslash_count() {
        assert!(needs_multiline_continuation("status \\"));
        assert!(!needs_multiline_continuation("status \\\\"));
    }

    #[test]
    fn eof_with_pending_multiline_sets_usage_error_state() {
        let (mut session, _) = startup_repl("", None);
        let _ = execute_repl_input(&mut session, ReplInput::Line("status \\".to_string()))
            .expect("line should set multiline pending");
        let _ = execute_repl_input(&mut session, ReplInput::Eof).expect("eof should exit cleanly");
        assert_eq!(session.last_exit_code, 2);
        assert_eq!(session.commands_executed, 1);
        assert!(session.last_error.as_deref().unwrap_or_default().contains("pending multiline"));
    }

    #[test]
    fn meta_exit_with_extra_args_is_invalid() {
        let (mut session, _) = startup_repl("", None);
        let result = execute_repl_line(&mut session, ":exit now");
        assert!(matches!(result, Err(ReplError::InvalidMetaCommand(_))));
        assert_eq!(session.last_exit_code, 2);
    }

    #[test]
    fn multiline_buffer_has_deterministic_upper_bound() {
        let (mut session, _) = startup_repl("", None);
        let oversized = format!("{}\\", "x".repeat(REPL_MULTILINE_BUFFER_MAX_CHARS + 1));
        let result = execute_repl_line(&mut session, &oversized);
        assert!(matches!(result, Err(ReplError::InvalidCommandInput(_))));
        assert_eq!(session.last_exit_code, 2);
    }
}
