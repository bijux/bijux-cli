use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use bijux_cli_contracts::{
    CommandPath, ErrorEnvelopeV1, Namespace, OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat,
    PrettyMode,
};
use bijux_cli_core::kernel::{
    DiagnosticsHook, Handler, LifecycleHook, SyncHandler, assemble_context, build_intent_from_argv,
    execute_pipeline,
};
use bijux_cli_output::{EmitterConfig, OutputStream as EmitStream, emit_error, emit_success};
use bijux_cli_routing::parser::{parse_intent, root_command};
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use serde_json::json;

use crate::history::push_history;
use crate::types::{META_PREFIX, ReplError, ReplEvent, ReplFrame, ReplInput, ReplSession, ReplStream};

struct ReplHandler;

impl SyncHandler for ReplHandler {
    fn execute(
        &self,
        ctx: &bijux_cli_core::kernel::ExecutionContext,
    ) -> Result<serde_json::Value, ErrorEnvelopeV1> {
        Ok(json!({
            "status": "ok",
            "route": ctx.intent.command_path.join(" "),
            "repl": true,
            "trace": ctx.trace_mode,
        }))
    }
}

fn parse_shell_tokens(input: &str) -> Vec<String> {
    shlex::split(input)
        .unwrap_or_else(|| input.split_whitespace().map(ToString::to_string).collect())
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
                content: if body.ends_with('\n') {
                    body
                } else {
                    format!("{body}\n")
                },
            })))
        }
        "set" if tokens.len() >= 3 => {
            match (tokens[1].as_str(), tokens[2].as_str()) {
                ("trace", "on") => session.trace_mode = true,
                ("trace", "off") => session.trace_mode = false,
                ("quiet", "on") => session.policy.quiet = true,
                ("quiet", "off") => session.policy.quiet = false,
                ("format", "json") => session.policy.output_format = OutputFormat::Json,
                ("format", "yaml") => session.policy.output_format = OutputFormat::Yaml,
                ("format", "text") => session.policy.output_format = OutputFormat::Text,
                _ => return Err(ReplError::InvalidMetaCommand(line.to_string())),
            }

            Ok(ReplEvent::Continue(Some(ReplFrame {
                stream: ReplStream::Stdout,
                content: "ok\n".to_string(),
            })))
        }
        "plugin" if tokens.len() >= 2 && tokens[1] == "reload" => {
            if !session.plugin_reload_safe {
                return Err(ReplError::PluginReloadUnsafe);
            }
            Ok(ReplEvent::Continue(Some(ReplFrame {
                stream: ReplStream::Stdout,
                content: "plugins reloaded\n".to_string(),
            })))
        }
        "exit" | "quit" => Ok(ReplEvent::Exit(None)),
        _ => Err(ReplError::InvalidMetaCommand(line.to_string())),
    }
}

fn emit_with_policy(
    session: &ReplSession,
    emission: bijux_cli_core::kernel::Emission,
) -> Result<Option<ReplFrame>, ReplError> {
    let config = EmitterConfig {
        format: session.policy.output_format,
        pretty: session.policy.pretty_mode == PrettyMode::Pretty,
        color: session.policy.color_mode,
        log_level: session.policy.log_level,
        quiet: session.policy.quiet,
        no_color: true,
    };

    let value = emission.payload;
    if emission.stream == bijux_cli_core::kernel::OutputStream::Stdout {
        let envelope = serde_json::from_value(value).unwrap_or_else(|_| OutputEnvelopeV1 {
            status: "ok".to_string(),
            data: json!({"repl": true}),
            meta: OutputEnvelopeMetaV1 {
                version: "v1".to_string(),
                command: CommandPath {
                    segments: vec![Namespace("repl".to_string())],
                },
                timestamp: "1970-01-01T00:00:00Z".to_string(),
            },
        });

        return Ok(emit_success(&envelope, config)?.map(|rendered| ReplFrame {
            stream: match rendered.stream {
                EmitStream::Stdout => ReplStream::Stdout,
                EmitStream::Stderr => ReplStream::Stderr,
            },
            content: rendered.content,
        }));
    }

    let envelope = serde_json::from_value(value).unwrap_or_else(|_| ErrorEnvelopeV1 {
        status: "error".to_string(),
        error: bijux_cli_contracts::ErrorPayloadV1 {
            code: "repl_error".to_string(),
            message: "REPL emission parsing failed".to_string(),
            category: "internal".to_string(),
            details: None,
        },
        meta: OutputEnvelopeMetaV1 {
            version: "v1".to_string(),
            command: CommandPath {
                segments: vec![Namespace("repl".to_string())],
            },
            timestamp: "1970-01-01T00:00:00Z".to_string(),
        },
    });

    let rendered = emit_error(&envelope, config)?;
    Ok(Some(ReplFrame {
        stream: match rendered.stream {
            EmitStream::Stdout => ReplStream::Stdout,
            EmitStream::Stderr => ReplStream::Stderr,
        },
        content: rendered.content,
    }))
}

/// Execute one REPL input event with interrupt/EOF-safe behavior.
pub fn execute_repl_input(session: &mut ReplSession, input: ReplInput) -> Result<ReplEvent, ReplError> {
    match input {
        ReplInput::Interrupt => {
            session.pending_multiline = None;
            session.last_exit_code = 130;
            Ok(ReplEvent::Interrupted(ReplFrame {
                stream: ReplStream::Stderr,
                content: "Interrupted\n".to_string(),
            }))
        }
        ReplInput::Eof => Ok(ReplEvent::Exit(None)),
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

            let parsed = match parse_intent(&argv) {
                Ok(value) => value,
                Err(error) => {
                    session.last_error = Some(error.to_string());
                    return Err(ReplError::Parser(error));
                }
            };
            let mut registry = RouteRegistry::default();
            let _ = registry.register_plugin_namespace("community");
            let target = match registry.resolve(&parsed.normalized_path) {
                Ok(value) => value,
                Err(error) => {
                    session.last_error = Some(error.to_string());
                    return Err(ReplError::Route(error));
                }
            };

            let intent = build_intent_from_argv(&argv);
            let context = assemble_context(
                intent,
                session.policy.clone(),
                None,
                Arc::new(AtomicBool::new(false)),
                session.trace_mode,
            );

            let handler = match target {
                RouteTarget::BuiltIn | RouteTarget::Plugin(_) => Handler::Sync(Box::new(ReplHandler)),
            };

            let diagnostics: Vec<Arc<dyn DiagnosticsHook>> = Vec::new();
            let lifecycle: Vec<Arc<dyn LifecycleHook>> = Vec::new();
            let result = execute_pipeline(&context, &handler, &diagnostics, &lifecycle)
                .map_err(|error| {
                    session.last_error = Some(format!("{error:?}"));
                    ReplError::Kernel(error)
                })?;

            session.commands_executed += 1;
            session.last_exit_code = result.exit_code as i32;

            let frame = match result.emission {
                Some(emission) => emit_with_policy(session, emission)?,
                None => None,
            };

            if let Some(trace) = result.trace {
                session.last_error = Some(format!("trace:{}", trace.invocation_id));
            }

            Ok(ReplEvent::Continue(frame))
        }
    }
}

/// Backward-compatible one-line execution adapter.
pub fn execute_repl_line(session: &mut ReplSession, line: &str) -> Result<Option<ReplFrame>, ReplError> {
    match execute_repl_input(session, ReplInput::Line(line.to_string()))? {
        ReplEvent::Continue(frame) => Ok(frame),
        ReplEvent::Exit(frame) => Ok(frame),
        ReplEvent::Interrupted(frame) => Ok(Some(frame)),
    }
}
