//! Execution kernel and lifecycle pipeline for Rust bijux-cli.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bijux_cli_contracts::{
    ColorMode, CommandPath, DiagnosticRecord, ErrorEnvelopeV1, ExecutionPolicy, ExitCode,
    GlobalFlags, InvocationEvent, InvocationTrace, LogLevel, Namespace, OutputEnvelopeMetaV1,
    OutputEnvelopeV1, OutputFormat, PrettyMode,
};
use serde_json::{json, Value};

/// Lifecycle stages executed by the kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleStage {
    /// Process bootstrap and runtime setup.
    Bootstrap,
    /// Command intent construction.
    BuildIntent,
    /// Policy resolution.
    ResolvePolicy,
    /// Context assembly.
    AssembleContext,
    /// Dispatch and execute handler.
    Dispatch,
    /// Emission stage.
    Emit,
    /// Exit mapping stage.
    ExitMap,
}

/// Execution intent built from argv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionIntent {
    /// Command path segments.
    pub command_path: Vec<String>,
    /// Parsed global flags.
    pub global_flags: GlobalFlags,
    /// Raw args after command path.
    pub args: Vec<String>,
}

/// Layered policy inputs from env/config/defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyInputs {
    /// Environment-provided policy values.
    pub env: GlobalFlags,
    /// Config-provided policy values.
    pub config: GlobalFlags,
    /// Hard defaults.
    pub defaults: GlobalFlags,
}

/// Execution context for handlers and stages.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Intent for the current invocation.
    pub intent: ExecutionIntent,
    /// Effective policy after precedence.
    pub policy: ExecutionPolicy,
    /// Timeout budget for handler execution.
    pub timeout: Option<Duration>,
    /// Cancellation token.
    pub cancelled: Arc<AtomicBool>,
    /// Structured route/policy/emission trace mode.
    pub trace_mode: bool,
}

/// Emission stream target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    /// Standard output stream.
    Stdout,
    /// Standard error stream.
    Stderr,
}

/// Emission produced by handler pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Emission {
    /// Output stream target.
    pub stream: OutputStream,
    /// Structured payload.
    pub payload: Value,
}

/// Handler outcome before emission mapping.
#[derive(Debug, Clone, PartialEq)]
pub enum HandlerOutcome {
    /// Successful payload.
    Success(Value),
    /// Error envelope payload.
    Error(ErrorEnvelopeV1),
}

/// Final execution result.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    /// Exit code.
    pub exit_code: ExitCode,
    /// Emission if any.
    pub emission: Option<Emission>,
    /// Optional trace payload.
    pub trace: Option<InvocationTrace>,
}

/// Diagnostic hook for structured internal telemetry.
pub trait DiagnosticsHook: Send + Sync {
    /// Observe a single record emitted by kernel stages.
    fn record(&self, record: DiagnosticRecord);
}

/// Lifecycle hook for plugin/repl boundaries.
pub trait LifecycleHook: Send + Sync {
    /// Invoked when plugin loading starts.
    fn on_plugin_load(&self) {}
    /// Invoked when REPL starts.
    fn on_repl_start(&self) {}
    /// Invoked when REPL shutdown starts.
    fn on_repl_shutdown(&self) {}
}

/// Sync handler abstraction.
pub trait SyncHandler: Send + Sync {
    /// Execute synchronously.
    fn execute(&self, ctx: &ExecutionContext) -> Result<Value, ErrorEnvelopeV1>;
}

/// Async handler abstraction.
pub trait AsyncHandler: Send + Sync {
    /// Execute asynchronously.
    fn execute_async(
        &self,
        ctx: &ExecutionContext,
    ) -> Pin<Box<dyn Future<Output = Result<Value, ErrorEnvelopeV1>> + Send + '_>>;
}

/// Unified handler variant supporting sync and async.
pub enum Handler {
    /// Sync handler variant.
    Sync(Box<dyn SyncHandler>),
    /// Async handler variant.
    Async(Box<dyn AsyncHandler>),
}

/// Kernel-level execution failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// Invocation was cancelled.
    Cancelled,
    /// Invocation exceeded configured timeout.
    Timeout,
}

/// Build intent from argv (simple kernel parser).
#[must_use]
pub fn build_intent_from_argv(argv: &[String]) -> ExecutionIntent {
    // PARITY-PARTIAL: This lightweight argv parser exists only for kernel-local tests.
    // Final parity paths should use routing parser intent output.
    let mut command_path = Vec::new();
    let mut args = Vec::new();

    let mut i = 1;
    while i < argv.len() {
        let token = &argv[i];
        if token.starts_with('-') {
            i += 1;
            continue;
        }

        command_path.push(token.clone());
        i += 1;
        break;
    }

    while i < argv.len() {
        args.push(argv[i].clone());
        i += 1;
    }

    let flags = GlobalFlags {
        output_format: None,
        pretty_mode: None,
        color_mode: None,
        log_level: None,
        quiet: argv.iter().any(|v| v == "--quiet" || v == "-q"),
        include_runtime: argv.iter().any(|v| v == "--trace"),
    };

    ExecutionIntent { command_path, global_flags: flags, args }
}

fn choose_opt<T: Copy>(a: Option<T>, b: Option<T>, c: Option<T>) -> Option<T> {
    a.or(b).or(c)
}

/// Resolve effective policy using flags -> env -> config -> defaults precedence.
#[must_use]
pub fn resolve_policy(intent: &ExecutionIntent, inputs: &PolicyInputs) -> ExecutionPolicy {
    let output_format = choose_opt(
        intent.global_flags.output_format,
        inputs.env.output_format,
        choose_opt(inputs.config.output_format, inputs.defaults.output_format, None),
    )
    .unwrap_or(OutputFormat::Json);

    let pretty_mode = choose_opt(
        intent.global_flags.pretty_mode,
        inputs.env.pretty_mode,
        choose_opt(inputs.config.pretty_mode, inputs.defaults.pretty_mode, None),
    )
    .unwrap_or(PrettyMode::Pretty);

    let color_mode = choose_opt(
        intent.global_flags.color_mode,
        inputs.env.color_mode,
        choose_opt(inputs.config.color_mode, inputs.defaults.color_mode, None),
    )
    .unwrap_or(ColorMode::Auto);

    let mut log_level = choose_opt(
        intent.global_flags.log_level,
        inputs.env.log_level,
        choose_opt(inputs.config.log_level, inputs.defaults.log_level, None),
    )
    .unwrap_or(LogLevel::Info);

    let quiet = intent.global_flags.quiet || inputs.env.quiet || inputs.config.quiet;
    if quiet {
        log_level = LogLevel::Error;
    }

    ExecutionPolicy {
        output_format,
        pretty_mode,
        color_mode,
        log_level,
        quiet,
        include_runtime: intent.global_flags.include_runtime || inputs.env.include_runtime,
    }
}

/// Assemble execution context.
#[must_use]
#[allow(dead_code)]
pub(crate) fn assemble_context(
    intent: ExecutionIntent,
    policy: ExecutionPolicy,
    timeout: Option<Duration>,
    cancelled: Arc<AtomicBool>,
    trace_mode: bool,
) -> ExecutionContext {
    ExecutionContext { intent, policy, timeout, cancelled, trace_mode }
}

#[allow(dead_code)]
fn success_meta(ctx: &ExecutionContext) -> OutputEnvelopeMetaV1 {
    OutputEnvelopeMetaV1 {
        version: "v1".to_string(),
        command: CommandPath {
            segments: ctx.intent.command_path.iter().map(|v| Namespace(v.clone())).collect(),
        },
        timestamp: "1970-01-01T00:00:00Z".to_string(),
    }
}

#[allow(dead_code)]
fn map_outcome_to_emission(outcome: HandlerOutcome, quiet: bool) -> Option<Emission> {
    if quiet {
        return None;
    }

    match outcome {
        HandlerOutcome::Success(payload) => {
            Some(Emission { stream: OutputStream::Stdout, payload })
        }
        HandlerOutcome::Error(err) => Some(Emission {
            stream: OutputStream::Stderr,
            payload: serde_json::to_value(err).expect("error envelope must serialize"),
        }),
    }
}

#[allow(dead_code)]
fn map_outcome_to_exit(outcome: &HandlerOutcome) -> ExitCode {
    match outcome {
        HandlerOutcome::Success(_) => ExitCode::Success,
        HandlerOutcome::Error(err) => map_error_category_to_exit(&err.error.category),
    }
}

/// Map stable error category to stable exit code contract.
#[must_use]
pub fn map_error_category_to_exit(category: &str) -> ExitCode {
    match category {
        "usage" | "validation" => ExitCode::Usage,
        "plugin" | "internal" => ExitCode::Error,
        _ => ExitCode::Error,
    }
}

#[allow(dead_code)]
fn is_fast_path(intent: &ExecutionIntent) -> bool {
    matches!(
        intent.command_path.as_slice(),
        [one] if one == "help" || one == "version" || one == "completion"
    )
}

/// Execute unified sync/async handler pipeline with lifecycle and diagnostics hooks.
#[allow(dead_code)]
pub(crate) fn execute_pipeline(
    ctx: &ExecutionContext,
    handler: &Handler,
    diagnostics: &[Arc<dyn DiagnosticsHook>],
    lifecycle: &[Arc<dyn LifecycleHook>],
) -> Result<ExecutionResult, KernelError> {
    let started_at = Instant::now();

    let mut trace_events = Vec::<InvocationEvent>::new();

    for hook in diagnostics {
        hook.record(DiagnosticRecord {
            id: "bootstrap".to_string(),
            severity: "info".to_string(),
            message: format!("stage={:?}", LifecycleStage::Bootstrap),
            fields: BTreeMap::new(),
        });
    }

    if ctx.cancelled.load(Ordering::SeqCst) {
        return Err(KernelError::Cancelled);
    }

    if is_fast_path(&ctx.intent) {
        // PARITY-PARTIAL: fast-path currently emits a generic payload; full parity requires
        // command-specific payloads for help/version/completion.
        let payload = OutputEnvelopeV1 {
            status: "ok".to_string(),
            data: json!({"fast_path": true}),
            meta: success_meta(ctx),
        };
        let outcome = HandlerOutcome::Success(
            serde_json::to_value(payload).expect("success envelope must serialize"),
        );
        let exit_code = map_outcome_to_exit(&outcome);
        let emission = map_outcome_to_emission(outcome, ctx.policy.quiet);
        return Ok(ExecutionResult {
            exit_code,
            emission,
            trace: if ctx.trace_mode {
                Some(InvocationTrace {
                    invocation_id: "trace-fast-path".to_string(),
                    command: CommandPath {
                        segments: ctx
                            .intent
                            .command_path
                            .iter()
                            .map(|v| Namespace(v.clone()))
                            .collect(),
                    },
                    policy: ctx.policy.clone(),
                    events: vec![InvocationEvent {
                        timestamp: "1970-01-01T00:00:00Z".to_string(),
                        name: "fast-path".to_string(),
                        payload: BTreeMap::new(),
                    }],
                })
            } else {
                None
            },
        });
    }

    if ctx.intent.command_path.first().is_some_and(|v| v == "plugins") {
        for hook in lifecycle {
            hook.on_plugin_load();
        }
    }
    if ctx.intent.command_path.first().is_some_and(|v| v == "repl") {
        for hook in lifecycle {
            hook.on_repl_start();
        }
    }

    let outcome = match handler {
        Handler::Sync(sync_handler) => match sync_handler.execute(ctx) {
            Ok(payload) => HandlerOutcome::Success(payload),
            Err(err) => HandlerOutcome::Error(err),
        },
        Handler::Async(async_handler) => {
            let result = futures::executor::block_on(async_handler.execute_async(ctx));
            match result {
                Ok(payload) => HandlerOutcome::Success(payload),
                Err(err) => HandlerOutcome::Error(err),
            }
        }
    };

    if let Some(limit) = ctx.timeout {
        if started_at.elapsed() > limit {
            return Err(KernelError::Timeout);
        }
    }

    if ctx.cancelled.load(Ordering::SeqCst) {
        return Err(KernelError::Cancelled);
    }

    if ctx.intent.command_path.first().is_some_and(|v| v == "repl") {
        for hook in lifecycle {
            hook.on_repl_shutdown();
        }
    }

    trace_events.push(InvocationEvent {
        timestamp: "1970-01-01T00:00:00Z".to_string(),
        name: "dispatch".to_string(),
        payload: BTreeMap::from([
            ("command".to_string(), json!(ctx.intent.command_path.join(" "))),
            ("emission".to_string(), json!("mapped")),
        ]),
    });

    let exit_code = map_outcome_to_exit(&outcome);
    let emission = map_outcome_to_emission(outcome, ctx.policy.quiet);

    Ok(ExecutionResult {
        exit_code,
        emission,
        trace: if ctx.trace_mode {
            Some(InvocationTrace {
                invocation_id: "trace-1".to_string(),
                command: CommandPath {
                    segments: ctx
                        .intent
                        .command_path
                        .iter()
                        .map(|v| Namespace(v.clone()))
                        .collect(),
                },
                policy: ctx.policy.clone(),
                events: trace_events,
            })
        } else {
            None
        },
    })
}

#[cfg(test)]
#[path = "kernel_pipeline_tests.rs"]
mod kernel_pipeline_tests;
