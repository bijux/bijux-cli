#![forbid(unsafe_code)]
//! REPL orchestration boundaries.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

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
/// REPL startup latency budget in milliseconds.
pub const REPL_STARTUP_LATENCY_BUDGET_MS: u128 = 50;
/// REPL memory budget in bytes.
pub const REPL_MEMORY_BUDGET_BYTES: usize = 2 * 1024 * 1024;

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
    /// Whether plugin reload command is allowed.
    pub plugin_reload_safe: bool,
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
    /// History replay index was invalid.
    #[error("history index out of bounds: {0}")]
    HistoryIndexOutOfBounds(usize),
    /// Plugin reload is blocked by safety policy.
    #[error("plugin reload is disabled by safety policy")]
    PluginReloadUnsafe,
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
        plugin_reload_safe: false,
    };

    let startup = ReplStartupContract {
        prompt,
        include_profile_context: !profile.is_empty(),
        policy,
    };

    (session, startup)
}

/// Startup REPL with startup diagnostics for preflight issues.
#[must_use]
pub fn startup_repl_with_diagnostics(
    profile: &str,
    prompt: Option<&str>,
    broken_plugins: &[&str],
) -> (ReplSession, ReplStartupContract, Vec<String>) {
    let (session, startup) = startup_repl(profile, prompt);
    let diagnostics = broken_plugins
        .iter()
        .map(|namespace| format!("plugin {namespace} is broken and will be skipped"))
        .collect();
    (session, startup, diagnostics)
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
    let mut entries: Vec<String> = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => {
            session.last_error = Some("history file is malformed; history reset".to_string());
            Vec::new()
        }
    };
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

/// Replay a command from history by index.
pub fn replay_history_command(
    session: &mut ReplSession,
    index: usize,
) -> Result<Option<ReplFrame>, ReplError> {
    let command = session
        .history
        .get(index)
        .cloned()
        .ok_or(ReplError::HistoryIndexOutOfBounds(index))?;
    execute_repl_line(session, &command)
}

/// Return last error message captured by REPL session.
#[must_use]
pub fn inspect_last_error(session: &ReplSession) -> Option<String> {
    session.last_error.clone()
}

/// Dump structured REPL diagnostics.
#[must_use]
pub fn session_diagnostics_dump(session: &ReplSession) -> String {
    let payload = json!({
        "session_id": session.session_id,
        "commands_executed": session.commands_executed,
        "last_exit_code": session.last_exit_code,
        "trace_mode": session.trace_mode,
        "history_size": session.history.len(),
        "history_limit": session.history_limit,
        "plugin_completion_hooks": session.plugin_completion_hooks.keys().collect::<Vec<_>>(),
        "last_error": session.last_error,
    });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    )
}

/// Render stable REPL command reference text.
#[must_use]
pub fn render_repl_command_reference() -> String {
    let lines = [
        "REPL Commands",
        "status",
        "doctor",
        "version",
        ":help <command>",
        ":set trace on|off",
        ":set quiet on|off",
        ":set format json|yaml|text",
        ":plugin reload",
        ":exit",
    ];
    format!("{}\n", lines.join("\n"))
}

/// Approximate REPL session memory use in bytes.
#[must_use]
pub fn estimated_session_memory_bytes(session: &ReplSession) -> usize {
    session.prompt.len()
        + session.profile.len()
        + session.history.iter().map(String::len).sum::<usize>()
        + session
            .plugin_completion_hooks
            .iter()
            .map(|(k, v)| k.len() + v.iter().map(String::len).sum::<usize>())
            .sum::<usize>()
        + 1024
}

/// Benchmark average startup latency over N iterations.
#[must_use]
pub fn benchmark_startup_latency(iterations: usize) -> Duration {
    let runs = iterations.max(1);
    let started = Instant::now();
    for _ in 0..runs {
        let _ = startup_repl("benchmark", None);
    }
    let total = started.elapsed();
    Duration::from_nanos((total.as_nanos() / runs as u128) as u64)
}

/// Check REPL runtime budgets.
#[must_use]
pub fn check_repl_budgets(session: &ReplSession, startup_avg: Duration) -> Vec<String> {
    let mut warnings = Vec::new();
    if startup_avg.as_millis() > REPL_STARTUP_LATENCY_BUDGET_MS {
        warnings.push(format!(
            "startup latency {}ms exceeded {}ms budget",
            startup_avg.as_millis(),
            REPL_STARTUP_LATENCY_BUDGET_MS
        ));
    }

    let estimated = estimated_session_memory_bytes(session);
    if estimated > REPL_MEMORY_BUDGET_BYTES {
        warnings.push(format!(
            "estimated memory {} bytes exceeded {} bytes budget",
            estimated, REPL_MEMORY_BUDGET_BYTES
        ));
    }
    warnings
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
                let outcome = handle_meta_command(session, &final_line);
                if let Err(error) = &outcome {
                    session.last_error = Some(error.to_string());
                }
                return outcome;
            }

            push_history(session, &final_line);

            let tokenized = parse_shell_tokens(&final_line);
            let argv: Vec<String> = std::iter::once("bijux".to_string()).chain(tokenized).collect();

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

    #[test]
    fn supports_history_replay_and_last_error_inspection() {
        let (mut session, _) = startup_repl("default", None);
        let _ = execute_repl_line(&mut session, "status").expect("status run");
        let replayed = replay_history_command(&mut session, 0).expect("replay should work");
        assert!(replayed.is_some());

        let error = execute_repl_input(&mut session, ReplInput::Line(":plugin reload".to_string()))
            .expect_err("reload should be blocked by default");
        assert!(matches!(error, ReplError::PluginReloadUnsafe));
        assert!(inspect_last_error(&session).is_some());
    }

    #[test]
    fn provides_session_diagnostics_dump_and_reference_text() {
        let (mut session, _) = startup_repl("default", None);
        let _ = execute_repl_line(&mut session, "status").expect("status run");
        let dump = session_diagnostics_dump(&session);
        assert!(dump.contains("commands_executed"));

        let reference = render_repl_command_reference();
        assert!(reference.contains(":help <command>"));
    }

    #[test]
    fn handles_history_corruption_without_failing_startup() {
        let (mut session, _) = startup_repl("default", None);
        let path = temp_history_file();
        fs::write(&path, "{broken-history").expect("write malformed history");
        configure_history(&mut session, Some(path), true, 100);
        load_history(&mut session).expect("corrupt history should be tolerated");
        assert!(session.history.is_empty());
        assert!(session.last_error.is_some());
    }

    #[test]
    fn supports_interactive_json_yaml_and_text_modes() {
        let (mut session, _) = startup_repl("default", None);

        let _ = execute_repl_input(&mut session, ReplInput::Line(":set format json".to_string()))
            .expect("json mode");
        let json_frame = execute_repl_line(&mut session, "status").expect("json line");
        assert!(json_frame.expect("frame").content.contains('{'));

        let _ = execute_repl_input(&mut session, ReplInput::Line(":set format yaml".to_string()))
            .expect("yaml mode");
        let yaml_frame = execute_repl_line(&mut session, "status").expect("yaml line");
        assert!(yaml_frame.expect("frame").content.contains("status:"));

        let _ = execute_repl_input(&mut session, ReplInput::Line(":set format text".to_string()))
            .expect("text mode");
        let text_frame = execute_repl_line(&mut session, "status").expect("text line");
        assert!(text_frame.expect("frame").content.contains("status"));
    }

    #[test]
    fn starts_with_and_without_plugin_preflight_diagnostics() {
        let (_session, _startup) = startup_repl("default", None);
        let (_session2, _startup2, diagnostics) =
            startup_repl_with_diagnostics("default", None, &["community"]);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn reports_budget_warnings_when_limits_are_exceeded() {
        let (mut session, _) = startup_repl("default", None);
        session.history = vec!["x".repeat(REPL_MEMORY_BUDGET_BYTES)];
        let startup_avg = Duration::from_millis((REPL_STARTUP_LATENCY_BUDGET_MS + 1) as u64);
        let warnings = check_repl_budgets(&session, startup_avg);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn benchmark_startup_latency_runs() {
        let latency = benchmark_startup_latency(5);
        assert!(latency.as_nanos() > 0);
    }
}
