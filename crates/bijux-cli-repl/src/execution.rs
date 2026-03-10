use bijux_cli_contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use bijux_cli_core::app::run_app;
use bijux_cli_routing::parser::root_command;

use crate::history::push_history;
use crate::types::{
    ReplError, ReplEvent, ReplFrame, ReplInput, ReplSession, ReplStream, META_PREFIX,
};

fn parse_shell_tokens(input: &str) -> Vec<String> {
    shlex::split(input)
        .unwrap_or_else(|| input.split_whitespace().map(ToString::to_string).collect())
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
        _ => "json",
    }
}

/// Build argv using the same tokenization path REPL execution uses.
#[must_use]
pub fn repl_argv_from_line(line: &str) -> Vec<String> {
    let tokenized = parse_shell_tokens(line);
    std::iter::once("bijux".to_string()).chain(tokenized).collect()
}

fn needs_multiline_continuation(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.ends_with('\\')
}

fn render_meta_help(path: &[String]) -> String {
    let mut command = root_command();
    let mut curr = &mut command;
    for segment in path {
        if let Some(next) = curr.find_subcommand_mut(segment) {
            curr = next;
        } else {
            return format!("Unknown command for help: {}\n", path.join(" "));
        }
    }

    let mut bytes = Vec::new();
    if curr.write_long_help(&mut bytes).is_ok() {
        String::from_utf8(bytes).unwrap_or_else(|_| "Unable to render help\n".to_string())
    } else {
        "Unable to render help\n".to_string()
    }
}

fn handle_meta_command(session: &mut ReplSession, line: &str) -> Result<ReplEvent, ReplError> {
    let raw = line.trim_start_matches(META_PREFIX).trim();
    let tokens = parse_shell_tokens(raw);
    if tokens.is_empty() {
        return Err(ReplError::InvalidMetaCommand(line.to_string()));
    }

    match tokens[0].as_str() {
        "help" => {
            let body = render_meta_help(&tokens[1..]);
            Ok(ReplEvent::Continue(Some(ReplFrame {
                stream: ReplStream::Stdout,
                content: if body.ends_with('\n') { body } else { format!("{body}\n") },
            })))
        }
        "set" if tokens.len() >= 3 => {
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
        "exit" | "quit" => Ok(ReplEvent::Exit(None)),
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
            _ => "--pretty",
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
            _ => "never",
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

    argv.extend_from_slice(&line_argv[1..]);
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
            session.last_exit_code = 130;
            Ok(ReplEvent::Interrupted(ReplFrame {
                stream: ReplStream::Stderr,
                content: "Interrupted\n".to_string(),
            }))
        }
        ReplInput::Eof => {
            session.pending_multiline = None;
            Ok(ReplEvent::Exit(None))
        }
        ReplInput::Line(line) => {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return Ok(ReplEvent::Continue(None));
            }

            if needs_multiline_continuation(trimmed) {
                let chunk = trimmed.trim_end_matches('\\').trim_end();
                session.pending_multiline = Some(match session.pending_multiline.take() {
                    Some(existing) => format!("{existing} {chunk}"),
                    None => chunk.to_string(),
                });
                return Ok(ReplEvent::Continue(None));
            }

            let final_line = if let Some(existing) = session.pending_multiline.take() {
                format!("{existing} {trimmed}")
            } else {
                trimmed.to_string()
            };

            if final_line.starts_with(META_PREFIX) {
                let outcome = handle_meta_command(session, &final_line);
                if let Err(error) = &outcome {
                    session.last_error = Some(error.to_string());
                }
                return outcome;
            }

            push_history(session, &final_line);

            let argv = repl_argv_from_line(&final_line);

            let effective_argv = apply_session_policy_to_argv(session, &argv);
            let result = run_app(&effective_argv).map_err(|error| {
                session.last_error = Some(error.to_string());
                ReplError::Core(error.to_string())
            })?;

            session.commands_executed += 1;
            session.last_exit_code = result.exit_code;

            let frame = if !result.stdout.is_empty() {
                Some(ReplFrame { stream: ReplStream::Stdout, content: result.stdout })
            } else if !result.stderr.is_empty() {
                session.last_error = Some(result.stderr.clone());
                Some(ReplFrame { stream: ReplStream::Stderr, content: result.stderr })
            } else {
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
