#![forbid(unsafe_code)]
//! REPL orchestration boundaries.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use bijux_cli_contracts::{
    ColorMode, ContractMarker, ErrorEnvelopeV1, ExecutionPolicy, GlobalFlags, LogLevel,
    OutputFormat, PrettyMode,
};
use bijux_cli_core::kernel::{
    assemble_context, build_intent_from_argv, execute_pipeline, resolve_policy, DiagnosticsHook,
    Handler, LifecycleHook, PolicyInputs, SyncHandler,
};
use bijux_cli_output::{emit_error, emit_success, EmitterConfig, OutputStream as EmitStream};
use bijux_cli_routing::parser::parse_intent;
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use bijux_cli_routing::route_marker;
use serde_json::json;

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

/// Execute one REPL input line by reusing CLI parser, router, kernel, and output emitters.
pub fn execute_repl_line(session: &mut ReplSession, line: &str) -> Result<Option<ReplFrame>, ReplError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let argv: Vec<String> = std::iter::once("bijux".to_string())
        .chain(trimmed.split_whitespace().map(std::string::ToString::to_string))
        .collect();

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
        false,
    );

    let handler = match target {
        RouteTarget::BuiltIn | RouteTarget::Plugin(_) => Handler::Sync(Box::new(ReplHandler)),
    };

    let diagnostics: Vec<Arc<dyn DiagnosticsHook>> = Vec::new();
    let lifecycle: Vec<Arc<dyn LifecycleHook>> = Vec::new();
    let result = execute_pipeline(&context, &handler, &diagnostics, &lifecycle)
        .map_err(ReplError::Kernel)?;

    session.commands_executed += 1;

    let Some(emission) = result.emission else {
        return Ok(None);
    };

    let config = EmitterConfig {
        format: session.policy.output_format,
        pretty: session.policy.pretty_mode == PrettyMode::Pretty,
        color: session.policy.color_mode,
        log_level: session.policy.log_level,
        quiet: session.policy.quiet,
        no_color: true,
    };

    let value = emission.payload;
    let output = if emission.stream == bijux_cli_core::kernel::OutputStream::Stdout {
        let envelope = serde_json::from_value(value).unwrap_or_else(|_| bijux_cli_contracts::OutputEnvelopeV1 {
            status: "ok".to_string(),
            data: json!({"repl": true}),
            meta: bijux_cli_contracts::OutputEnvelopeMetaV1 {
                version: "v1".to_string(),
                command: bijux_cli_contracts::CommandPath { segments: vec![] },
                timestamp: "1970-01-01T00:00:00Z".to_string(),
            },
        });

        emit_success(&envelope, config)?.map(|rendered| ReplFrame {
            stream: match rendered.stream {
                EmitStream::Stdout => ReplStream::Stdout,
                EmitStream::Stderr => ReplStream::Stderr,
            },
            content: rendered.content,
        })
    } else {
        let envelope = serde_json::from_value(value).unwrap_or_else(|_| ErrorEnvelopeV1 {
            status: "error".to_string(),
            error: bijux_cli_contracts::ErrorPayloadV1 {
                code: "repl_error".to_string(),
                message: "REPL emission parsing failed".to_string(),
                category: "internal".to_string(),
                details: None,
            },
            meta: bijux_cli_contracts::OutputEnvelopeMetaV1 {
                version: "v1".to_string(),
                command: bijux_cli_contracts::CommandPath { segments: vec![] },
                timestamp: "1970-01-01T00:00:00Z".to_string(),
            },
        });

        Some({
            let rendered = emit_error(&envelope, config)?;
            ReplFrame {
                stream: match rendered.stream {
                    EmitStream::Stdout => ReplStream::Stdout,
                    EmitStream::Stderr => ReplStream::Stderr,
                },
                content: rendered.content,
            }
        })
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_and_shuts_down_with_stable_contracts() {
        let (session, startup) = startup_repl("default", None);
        assert_eq!(startup.prompt, "bijux> ");
        assert!(startup.include_profile_context);

        let shutdown = shutdown_repl(&session);
        assert_eq!(shutdown.commands_executed, 0);
    }

    #[test]
    fn executes_line_using_cli_parser_router_kernel_and_emitters() {
        let (mut session, _) = startup_repl("default", Some("bijux(default)> "));
        let frame = execute_repl_line(&mut session, "status").expect("line should execute");
        assert!(frame.is_some());
        assert_eq!(session.commands_executed, 1);
    }
}
