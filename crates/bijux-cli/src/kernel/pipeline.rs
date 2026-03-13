//! Execution kernel and lifecycle pipeline for Rust bijux-cli.

use std::collections::BTreeMap;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::contracts::diagnostics::{DiagnosticRecord, InvocationEvent, InvocationTrace};
use crate::contracts::{
    CommandPath, ErrorEnvelopeV1, ErrorPayloadV1, ExecutionPolicy, ExitCode, GlobalFlags,
    Namespace, OutputEnvelopeMetaV1, OutputEnvelopeV1,
};
use crate::shared::telemetry::{truncate_chars, MAX_COMMAND_FIELD_CHARS, MAX_TEXT_FIELD_CHARS};
use serde_json::{json, Value};

static INVOCATION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Lifecycle stages executed by the kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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
#[cfg_attr(not(test), allow(dead_code))]
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
        timestamp: event_timestamp(),
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

#[allow(dead_code)]
fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "unknown panic payload".to_string()
}

#[allow(dead_code)]
fn internal_kernel_error(ctx: &ExecutionContext, message: String) -> HandlerOutcome {
    HandlerOutcome::Error(ErrorEnvelopeV1::failure(
        ErrorPayloadV1 {
            code: "KERNEL_HANDLER_PANIC".to_string(),
            message: format!("kernel handler panicked: {message}"),
            category: "internal".to_string(),
            details: None,
        },
        success_meta(ctx),
    ))
}

#[allow(dead_code)]
fn catch_unwind_silent<T>(f: impl FnOnce() -> T) -> std::thread::Result<T> {
    static PANIC_HOOK_LOCK: Mutex<()> = Mutex::new(());
    let _guard = PANIC_HOOK_LOCK.lock().expect("panic hook lock poisoned");
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(previous_hook);
    result
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

fn unix_timestamp_millis() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}

fn event_timestamp() -> String {
    unix_timestamp_millis().to_string()
}

fn exit_code_kind(exit_code: ExitCode) -> &'static str {
    match exit_code {
        ExitCode::Success => "success",
        ExitCode::Usage => "usage",
        ExitCode::Encoding => "encoding",
        ExitCode::Aborted => "aborted",
        ExitCode::Error => "error",
    }
}

fn next_invocation_id() -> String {
    let seq = INVOCATION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("kernel-{}-{}-{seq}", std::process::id(), unix_timestamp_millis())
}

fn bounded_command(command: &str) -> (String, bool) {
    truncate_chars(command, MAX_COMMAND_FIELD_CHARS)
}

fn bounded_message(message: &str) -> (String, bool) {
    truncate_chars(message, MAX_TEXT_FIELD_CHARS)
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
        let command_joined = ctx.intent.command_path.join(" ");
        let (command, command_truncated) = bounded_command(&command_joined);
        for hook in diagnostics {
            hook.record(DiagnosticRecord {
                id: "kernel_cancelled_before_dispatch".to_string(),
                severity: "warning".to_string(),
                message: "execution cancelled before dispatch".to_string(),
                fields: BTreeMap::from([
                    ("command".to_string(), json!(command)),
                    ("command_truncated".to_string(), json!(command_truncated)),
                ]),
            });
        }
        return Err(KernelError::Cancelled);
    }

    trace_events.push(InvocationEvent {
        // Trace remains human-readable; diagnostics carry truncated metadata separately.
        timestamp: event_timestamp(),
        name: "bootstrap".to_string(),
        payload: BTreeMap::from([(
            "command".to_string(),
            json!(ctx.intent.command_path.join(" ")),
        )]),
    });

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
        let emission_stream = emission.as_ref().map(|item| match item.stream {
            OutputStream::Stdout => "stdout",
            OutputStream::Stderr => "stderr",
        });
        for hook in diagnostics {
            let command_joined = ctx.intent.command_path.join(" ");
            let (command, command_truncated) = bounded_command(&command_joined);
            hook.record(DiagnosticRecord {
                id: "kernel_dispatch_fast_path_completed".to_string(),
                severity: "info".to_string(),
                message: "fast-path command completed".to_string(),
                fields: BTreeMap::from([
                    ("command".to_string(), json!(command)),
                    ("command_truncated".to_string(), json!(command_truncated)),
                    ("exit_code".to_string(), json!(exit_code as i32)),
                    ("exit_kind".to_string(), json!(exit_code_kind(exit_code))),
                ]),
            });
        }
        return Ok(ExecutionResult {
            exit_code,
            emission,
            trace: if ctx.trace_mode {
                Some(InvocationTrace {
                    invocation_id: next_invocation_id(),
                    command: CommandPath {
                        segments: ctx
                            .intent
                            .command_path
                            .iter()
                            .map(|v| Namespace(v.clone()))
                            .collect(),
                    },
                    policy: ctx.policy.clone(),
                    events: vec![
                        InvocationEvent {
                            timestamp: event_timestamp(),
                            name: "dispatch.start".to_string(),
                            payload: BTreeMap::from([
                                ("command".to_string(), json!(ctx.intent.command_path.join(" "))),
                                ("mode".to_string(), json!("fast-path")),
                            ]),
                        },
                        InvocationEvent {
                            timestamp: event_timestamp(),
                            name: "dispatch.finish".to_string(),
                            payload: BTreeMap::from([
                                ("command".to_string(), json!(ctx.intent.command_path.join(" "))),
                                ("mode".to_string(), json!("fast-path")),
                                ("exit_code".to_string(), json!(exit_code as i32)),
                                ("exit_kind".to_string(), json!(exit_code_kind(exit_code))),
                                ("quiet".to_string(), json!(ctx.policy.quiet)),
                                ("emission".to_string(), json!(emission_stream)),
                                (
                                    "duration_ms".to_string(),
                                    json!(started_at.elapsed().as_millis()),
                                ),
                            ]),
                        },
                    ],
                })
            } else {
                None
            },
        });
    }

    if ctx.intent.command_path.first().is_some_and(|v| v == "plugins") {
        trace_events.push(InvocationEvent {
            timestamp: event_timestamp(),
            name: "lifecycle.plugin.load".to_string(),
            payload: BTreeMap::from([(
                "command".to_string(),
                json!(ctx.intent.command_path.join(" ")),
            )]),
        });
        for hook in lifecycle {
            hook.on_plugin_load();
        }
    }
    let is_repl_command = ctx.intent.command_path.first().is_some_and(|v| v == "repl");
    if is_repl_command {
        trace_events.push(InvocationEvent {
            timestamp: event_timestamp(),
            name: "lifecycle.repl.start".to_string(),
            payload: BTreeMap::from([(
                "command".to_string(),
                json!(ctx.intent.command_path.join(" ")),
            )]),
        });
        for hook in lifecycle {
            hook.on_repl_start();
        }
    }

    trace_events.push(InvocationEvent {
        timestamp: event_timestamp(),
        name: "dispatch.start".to_string(),
        payload: BTreeMap::from([
            ("command".to_string(), json!(ctx.intent.command_path.join(" "))),
            ("mode".to_string(), json!("standard")),
        ]),
    });

    let outcome = match handler {
        Handler::Sync(sync_handler) => match catch_unwind_silent(|| sync_handler.execute(ctx)) {
            Ok(Ok(payload)) => HandlerOutcome::Success(payload),
            Ok(Err(err)) => HandlerOutcome::Error(err),
            Err(payload) => {
                let panic_text = panic_message(payload);
                let command_joined = ctx.intent.command_path.join(" ");
                let (command, command_truncated) = bounded_command(&command_joined);
                let (panic_message_text, panic_message_truncated) = bounded_message(&panic_text);
                for hook in diagnostics {
                    hook.record(DiagnosticRecord {
                        id: "kernel_handler_panic".to_string(),
                        severity: "error".to_string(),
                        message: "sync handler panicked during dispatch".to_string(),
                        fields: BTreeMap::from([
                            ("command".to_string(), json!(command)),
                            ("command_truncated".to_string(), json!(command_truncated)),
                            ("panic_message".to_string(), json!(panic_message_text)),
                            ("panic_message_truncated".to_string(), json!(panic_message_truncated)),
                        ]),
                    });
                }
                internal_kernel_error(ctx, panic_text)
            }
        },
        Handler::Async(async_handler) => {
            match catch_unwind_silent(|| {
                futures::executor::block_on(async_handler.execute_async(ctx))
            }) {
                Ok(Ok(payload)) => HandlerOutcome::Success(payload),
                Ok(Err(err)) => HandlerOutcome::Error(err),
                Err(payload) => {
                    let panic_text = panic_message(payload);
                    let command_joined = ctx.intent.command_path.join(" ");
                    let (command, command_truncated) = bounded_command(&command_joined);
                    let (panic_message_text, panic_message_truncated) =
                        bounded_message(&panic_text);
                    for hook in diagnostics {
                        hook.record(DiagnosticRecord {
                            id: "kernel_handler_panic".to_string(),
                            severity: "error".to_string(),
                            message: "async handler panicked during dispatch".to_string(),
                            fields: BTreeMap::from([
                                ("command".to_string(), json!(command)),
                                ("command_truncated".to_string(), json!(command_truncated)),
                                ("panic_message".to_string(), json!(panic_message_text)),
                                (
                                    "panic_message_truncated".to_string(),
                                    json!(panic_message_truncated),
                                ),
                            ]),
                        });
                    }
                    internal_kernel_error(ctx, panic_text)
                }
            }
        }
    };

    if is_repl_command {
        trace_events.push(InvocationEvent {
            timestamp: event_timestamp(),
            name: "lifecycle.repl.shutdown".to_string(),
            payload: BTreeMap::from([(
                "command".to_string(),
                json!(ctx.intent.command_path.join(" ")),
            )]),
        });
        for hook in lifecycle {
            hook.on_repl_shutdown();
        }
    }

    if let Some(limit) = ctx.timeout {
        if started_at.elapsed() > limit {
            let command_joined = ctx.intent.command_path.join(" ");
            let (command, command_truncated) = bounded_command(&command_joined);
            for hook in diagnostics {
                hook.record(DiagnosticRecord {
                    id: "kernel_timeout_after_dispatch".to_string(),
                    severity: "error".to_string(),
                    message: format!(
                        "execution exceeded timeout budget of {}ms",
                        limit.as_millis()
                    ),
                    fields: BTreeMap::from([
                        ("command".to_string(), json!(command)),
                        ("command_truncated".to_string(), json!(command_truncated)),
                        ("timeout_ms".to_string(), json!(limit.as_millis())),
                        ("elapsed_ms".to_string(), json!(started_at.elapsed().as_millis())),
                    ]),
                });
            }
            return Err(KernelError::Timeout);
        }
    }

    if ctx.cancelled.load(Ordering::SeqCst) {
        let command_joined = ctx.intent.command_path.join(" ");
        let (command, command_truncated) = bounded_command(&command_joined);
        for hook in diagnostics {
            hook.record(DiagnosticRecord {
                id: "kernel_cancelled_after_dispatch".to_string(),
                severity: "warning".to_string(),
                message: "execution cancelled after dispatch".to_string(),
                fields: BTreeMap::from([
                    ("command".to_string(), json!(command)),
                    ("command_truncated".to_string(), json!(command_truncated)),
                ]),
            });
        }
        return Err(KernelError::Cancelled);
    }

    match &outcome {
        HandlerOutcome::Success(_) => {
            let command_joined = ctx.intent.command_path.join(" ");
            let (command, command_truncated) = bounded_command(&command_joined);
            for hook in diagnostics {
                hook.record(DiagnosticRecord {
                    id: "kernel_handler_outcome_success".to_string(),
                    severity: "info".to_string(),
                    message: "handler returned success payload".to_string(),
                    fields: BTreeMap::from([
                        ("command".to_string(), json!(command)),
                        ("command_truncated".to_string(), json!(command_truncated)),
                    ]),
                });
            }
        }
        HandlerOutcome::Error(err) => {
            let command_joined = ctx.intent.command_path.join(" ");
            let (command, command_truncated) = bounded_command(&command_joined);
            let (error_code, error_code_truncated) = bounded_message(&err.error.code);
            let (error_category, error_category_truncated) = bounded_message(&err.error.category);
            for hook in diagnostics {
                hook.record(DiagnosticRecord {
                    id: "kernel_handler_outcome_error".to_string(),
                    severity: "warning".to_string(),
                    message: "handler returned error payload".to_string(),
                    fields: BTreeMap::from([
                        ("command".to_string(), json!(command)),
                        ("command_truncated".to_string(), json!(command_truncated)),
                        ("error_category".to_string(), json!(error_category)),
                        ("error_category_truncated".to_string(), json!(error_category_truncated)),
                        ("error_code".to_string(), json!(error_code)),
                        ("error_code_truncated".to_string(), json!(error_code_truncated)),
                    ]),
                });
            }
        }
    }

    let exit_code = map_outcome_to_exit(&outcome);
    let outcome_error_category = match &outcome {
        HandlerOutcome::Success(_) => None::<String>,
        HandlerOutcome::Error(err) => Some(err.error.category.clone()),
    };
    let outcome_error_code = match &outcome {
        HandlerOutcome::Success(_) => None::<String>,
        HandlerOutcome::Error(err) => Some(err.error.code.clone()),
    };
    let emission = map_outcome_to_emission(outcome, ctx.policy.quiet);
    let emission_stream = emission.as_ref().map(|item| match item.stream {
        OutputStream::Stdout => "stdout",
        OutputStream::Stderr => "stderr",
    });

    trace_events.push(InvocationEvent {
        timestamp: event_timestamp(),
        name: "dispatch.finish".to_string(),
        payload: BTreeMap::from([
            ("command".to_string(), json!(ctx.intent.command_path.join(" "))),
            ("mode".to_string(), json!("standard")),
            ("exit_code".to_string(), json!(exit_code as i32)),
            ("exit_kind".to_string(), json!(exit_code_kind(exit_code))),
            ("quiet".to_string(), json!(ctx.policy.quiet)),
            ("error_category".to_string(), json!(outcome_error_category)),
            ("error_code".to_string(), json!(outcome_error_code)),
            ("emission".to_string(), json!(emission_stream)),
            ("duration_ms".to_string(), json!(started_at.elapsed().as_millis())),
        ]),
    });

    for hook in diagnostics {
        let command_joined = ctx.intent.command_path.join(" ");
        let (command, command_truncated) = bounded_command(&command_joined);
        hook.record(DiagnosticRecord {
            id: "kernel_dispatch_completed".to_string(),
            severity: "info".to_string(),
            message: "kernel dispatch finished".to_string(),
            fields: BTreeMap::from([
                ("command".to_string(), json!(command)),
                ("command_truncated".to_string(), json!(command_truncated)),
                ("exit_code".to_string(), json!(exit_code as i32)),
                ("exit_kind".to_string(), json!(exit_code_kind(exit_code))),
                ("duration_ms".to_string(), json!(started_at.elapsed().as_millis())),
            ]),
        });
    }

    Ok(ExecutionResult {
        exit_code,
        emission,
        trace: if ctx.trace_mode {
            Some(InvocationTrace {
                invocation_id: next_invocation_id(),
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
