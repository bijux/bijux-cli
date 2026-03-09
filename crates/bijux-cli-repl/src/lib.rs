#![forbid(unsafe_code)]
//! REPL orchestration boundaries.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use bijux_cli_contracts::{
    ColorMode, CommandPath, ContractMarker, ErrorEnvelopeV1, ExecutionPolicy, GlobalFlags,
    LogLevel, Namespace, OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat, PrettyMode,
};
use bijux_cli_core::kernel::{
    assemble_context, build_intent_from_argv, execute_pipeline, resolve_policy, DiagnosticsHook,
    Handler, LifecycleHook, PolicyInputs, SyncHandler,
};
use bijux_cli_output::{emit_error, emit_success, EmitterConfig, OutputStream as EmitStream};
use bijux_cli_routing::parser::{parse_intent, root_command};
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use bijux_cli_routing::route_marker;
use serde_json::json;

const META_PREFIX: char = ':';

/// Stable REPL startup contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplStartupContract {
    /// Prompt format string.
    pub prompt: String,
    /// Whether profile/context is displayed in prompt.
    pub include_profile_context: bool,
    /// Effective startup policy.
    pub policy: ExecutionPolicy,
}

/// Stable REPL shutdown contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplShutdownContract {
    /// Session id.
    pub session_id: String,
    /// Number of commands executed.
    pub commands_executed: usize,
}

/// REPL session model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplSession {
    /// Session identifier.
    pub session_id: String,
    /// Prompt displayed to user.
    pub prompt: String,
    /// Profile label shown in prompt.
    pub profile: String,
    /// Effective execution policy.
    pub policy: ExecutionPolicy,
    /// Command counter.
    pub commands_executed: usize,
    /// Last mapped exit code as integer.
    pub last_exit_code: i32,
    /// Trace mode toggle.
    pub trace_mode: bool,
    /// Persistent command history buffer.
    pub history: Vec<String>,
    /// Max history size.
    pub history_limit: usize,
    /// Whether history persistence is enabled.
    pub history_enabled: bool,
    /// History file location.
    pub history_file: Option<PathBuf>,
    /// Pending multiline input buffer.
    pub pending_multiline: Option<String>,
    /// Last observed error message.
    pub last_error: Option<String>,
    /// Plugin completion hooks by namespace.
    pub plugin_completion_hooks: BTreeMap<String, Vec<String>>,
}

/// REPL emission stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// Repl output frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplFrame {
    /// Stream target.
    pub stream: ReplStream,
    /// Serialized output.
    pub content: String,
}

/// Input event for interactive session loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplInput {
    /// Normal command line.
    Line(String),
    /// Ctrl-C interrupt event.
    Interrupt,
    /// EOF event.
    Eof,
}

/// Result of processing a REPL input event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplEvent {
    /// Keep session alive.
    Continue(Option<ReplFrame>),
    /// Exit session.
    Exit(Option<ReplFrame>),
    /// Interrupted command input.
    Interrupted(ReplFrame),
}

/// REPL runtime errors.
#[derive(Debug, thiserror::Error)]
pub enum ReplError {
    /// Command parser failed.
    #[error(transparent)]
    Parser(#[from] bijux_cli_routing::parser::ParseError),
    /// Routing failed.
    #[error(transparent)]
    Route(#[from] bijux_cli_routing::registry::RouteError),
    /// Kernel failed.
    #[error("kernel execution failed")]
    Kernel(bijux_cli_core::kernel::KernelError),
    /// Output encoding failed.
    #[error(transparent)]
    Emit(#[from] bijux_cli_output::EmitError),
    /// History serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// IO failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Invalid REPL command.
    #[error("invalid repl command: {0}")]
    InvalidMetaCommand(String),
}

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

/// Build REPL marker chained from routing state.
#[must_use]
pub fn repl_marker() -> ContractMarker {
    let mut marker = route_marker();
    marker.namespace = format!("{}:repl", marker.namespace);
    marker
}

/// Startup REPL session using the same policy precedence and routing registry as CLI.
#[must_use]
pub fn startup_repl(profile: &str, prompt: Option<&str>) -> (ReplSession, ReplStartupContract) {
    let defaults = GlobalFlags {
        output_format: Some(OutputFormat::Json),
        pretty_mode: Some(PrettyMode::Pretty),
        color_mode: Some(ColorMode::Never),
        log_level: Some(LogLevel::Info),
        quiet: false,
        include_runtime: false,
    };

    let policy = resolve_policy(
        &build_intent_from_argv(&["bijux".to_string(), "repl".to_string()]),
        &PolicyInputs { env: defaults.clone(), config: defaults.clone(), defaults },
    );

    let prompt = prompt.unwrap_or("bijux> ").to_string();
    let session = ReplSession {
        session_id: "repl-1".to_string(),
        prompt: prompt.clone(),
        profile: profile.to_string(),
        policy: policy.clone(),
        commands_executed: 0,
        last_exit_code: 0,
        trace_mode: false,
        history: Vec::new(),
        history_limit: 500,
        history_enabled: true,
        history_file: None,
        pending_multiline: None,
        last_error: None,
        plugin_completion_hooks: BTreeMap::new(),
    };

    let startup = ReplStartupContract {
        prompt,
        include_profile_context: !profile.is_empty(),
        policy,
    };

    (session, startup)
}

/// Shutdown REPL session and emit stable contract.
#[must_use]
pub fn shutdown_repl(session: &ReplSession) -> ReplShutdownContract {
    ReplShutdownContract {
        session_id: session.session_id.clone(),
        commands_executed: session.commands_executed,
    }
}

/// Configure history persistence behavior.
pub fn configure_history(
    session: &mut ReplSession,
    history_file: Option<PathBuf>,
    enabled: bool,
    limit: usize,
) {
    session.history_file = history_file;
    session.history_enabled = enabled;
    session.history_limit = limit.max(1);
}

/// Load history into the current session if enabled.
pub fn load_history(session: &mut ReplSession) -> Result<(), ReplError> {
    if !session.history_enabled {
        return Ok(());
    }
    let Some(path) = &session.history_file else {
        return Ok(());
    };
    if !path.exists() {
        return Ok(());
    }

    let text = fs::read_to_string(path)?;
    let mut entries: Vec<String> = serde_json::from_str(&text)?;
    if entries.len() > session.history_limit {
        entries = entries.split_off(entries.len() - session.history_limit);
    }
    session.history = entries;
    Ok(())
}

/// Flush history to persistent storage if enabled.
pub fn flush_history(session: &ReplSession) -> Result<(), ReplError> {
    if !session.history_enabled {
        return Ok(());
    }
    let Some(path) = &session.history_file else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(&session.history)?;
    fs::write(path, format!("{data}\n"))?;
    Ok(())
}

fn push_history(session: &mut ReplSession, command: &str) {
    if !session.history_enabled || command.is_empty() {
        return;
    }
    session.history.push(command.to_string());
    if session.history.len() > session.history_limit {
        let overflow = session.history.len() - session.history_limit;
        session.history.drain(0..overflow);
    }
}

/// Provide command completion candidates for built-ins and plugin hooks.
#[must_use]
pub fn completion_candidates(session: &ReplSession, prefix: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    let builtins = [
        "help", "version", "doctor", "repl", "completion", "inspect", "status", "config",
        "plugins", "dev", "history",
    ];
    for builtin in builtins {
        if builtin.starts_with(prefix) {
            suggestions.push(builtin.to_string());
        }
    }

    for namespace in session.plugin_completion_hooks.keys() {
        if namespace.starts_with(prefix) {
            suggestions.push(namespace.clone());
        }
    }

    for values in session.plugin_completion_hooks.values() {
        for value in values {
            if value.starts_with(prefix) {
                suggestions.push(value.clone());
            }
        }
    }

    suggestions.sort();
    suggestions.dedup();
    suggestions
}

/// Register plugin completion hook for a namespace.
pub fn register_plugin_completion_hook(
    session: &mut ReplSession,
    namespace: &str,
    suggestions: Vec<String>,
) {
    session.plugin_completion_hooks.insert(namespace.to_string(), suggestions);
}

fn parse_shell_tokens(input: &str) -> Vec<String> {
    shlex::split(input).unwrap_or_else(|| input.split_whitespace().map(ToString::to_string).collect())
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
                command: CommandPath { segments: vec![Namespace("repl".to_string())] },
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
            command: CommandPath { segments: vec![Namespace("repl".to_string())] },
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
                return handle_meta_command(session, &final_line);
            }

            push_history(session, &final_line);

            let tokenized = parse_shell_tokens(&final_line);
            let argv: Vec<String> = std::iter::once("bijux".to_string()).chain(tokenized).collect();

            let parsed = parse_intent(&argv)?;
            let mut registry = RouteRegistry::default();
            let _ = registry.register_plugin_namespace("community");
            let target = registry.resolve(&parsed.normalized_path)?;

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
            let result =
                execute_pipeline(&context, &handler, &diagnostics, &lifecycle).map_err(ReplError::Kernel)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_history_file() -> PathBuf {
        std::env::temp_dir().join("bijux-repl-history-test.json")
    }

    #[test]
    fn starts_and_shuts_down_with_stable_contracts() {
        let (session, startup) = startup_repl("default", None);
        assert_eq!(startup.prompt, "bijux> ");
        assert!(startup.include_profile_context);

        let shutdown = shutdown_repl(&session);
        assert_eq!(shutdown.commands_executed, 0);
    }

    #[test]
    fn supports_history_load_flush_cap_and_opt_out() {
        let (mut session, _) = startup_repl("default", None);
        let path = temp_history_file();
        configure_history(&mut session, Some(path.clone()), true, 2);

        let _ = execute_repl_line(&mut session, "status").expect("exec1");
        let _ = execute_repl_line(&mut session, "doctor").expect("exec2");
        let _ = execute_repl_line(&mut session, "version").expect("exec3");
        assert_eq!(session.history.len(), 2);

        flush_history(&session).expect("flush");

        let (mut loaded, _) = startup_repl("default", None);
        configure_history(&mut loaded, Some(path.clone()), true, 2);
        load_history(&mut loaded).expect("load");
        assert_eq!(loaded.history.len(), 2);

        configure_history(&mut loaded, Some(path), false, 2);
        loaded.history.clear();
        load_history(&mut loaded).expect("opt-out should skip loading");
        assert!(loaded.history.is_empty());
    }

    #[test]
    fn completion_includes_builtins_and_plugin_hooks() {
        let (mut session, _) = startup_repl("default", None);
        register_plugin_completion_hook(
            &mut session,
            "community",
            vec!["community status".to_string(), "community inspect".to_string()],
        );

        let values = completion_candidates(&session, "com");
        assert!(values.iter().any(|v| v == "completion"));
        assert!(values.iter().any(|v| v == "community"));
    }

    #[test]
    fn meta_commands_handle_help_and_runtime_switches() {
        let (mut session, _) = startup_repl("default", None);

        let help = execute_repl_input(&mut session, ReplInput::Line(":help status".to_string()))
            .expect("help should execute");
        assert!(matches!(help, ReplEvent::Continue(Some(_))));

        let _ = execute_repl_input(&mut session, ReplInput::Line(":set trace on".to_string()))
            .expect("trace switch");
        assert!(session.trace_mode);

        let _ = execute_repl_input(&mut session, ReplInput::Line(":set quiet on".to_string()))
            .expect("quiet switch");
        assert!(session.policy.quiet);

        let _ = execute_repl_input(&mut session, ReplInput::Line(":set format yaml".to_string()))
            .expect("format switch");
        assert_eq!(session.policy.output_format, OutputFormat::Yaml);
    }

    #[test]
    fn handles_interrupt_eof_and_multiline_flow() {
        let (mut session, _) = startup_repl("default", None);

        let interrupted = execute_repl_input(&mut session, ReplInput::Interrupt).expect("interrupt");
        assert!(matches!(interrupted, ReplEvent::Interrupted(_)));
        assert_eq!(session.last_exit_code, 130);

        let _ = execute_repl_input(&mut session, ReplInput::Line("status \\\\".to_string()))
            .expect("start multiline");
        assert!(session.pending_multiline.is_some());

        let line = execute_repl_input(&mut session, ReplInput::Line("--format json".to_string()))
            .expect("complete multiline");
        assert!(matches!(line, ReplEvent::Continue(_)));

        let eof = execute_repl_input(&mut session, ReplInput::Eof).expect("eof");
        assert!(matches!(eof, ReplEvent::Exit(_)));
    }

    #[test]
    fn executes_line_using_cli_parser_router_kernel_and_emitters() {
        let (mut session, _) = startup_repl("default", Some("bijux(default)> "));
        let frame = execute_repl_line(&mut session, "status").expect("line should execute");
        assert!(frame.is_some());
        assert_eq!(session.commands_executed, 1);
        assert_eq!(session.last_exit_code, 0);
    }
}
