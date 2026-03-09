#![forbid(unsafe_code)]
//! Full execution pipeline tests for the core runtime.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_plugin as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use clap as _;
use bijux_cli_contracts::{
    ColorMode, ExecutionPolicy, ExitCode, GlobalFlags, LogLevel, OutputFormat, PrettyMode,
};
use bijux_cli_core::kernel::{
    assemble_context, build_intent_from_argv, execute_pipeline, map_error_category_to_exit,
    resolve_policy, AsyncHandler, DiagnosticsHook, ExecutionIntent, Handler, KernelError,
    LifecycleHook, PolicyInputs, SyncHandler,
};
use futures::future;
use serde_json::json;

struct SyncOk;
impl SyncHandler for SyncOk {
    fn execute(
        &self,
        _ctx: &bijux_cli_core::kernel::ExecutionContext,
    ) -> Result<serde_json::Value, bijux_cli_contracts::ErrorEnvelopeV1> {
        Ok(json!({"ok": true}))
    }
}

struct AsyncOk;
impl AsyncHandler for AsyncOk {
    fn execute_async(
        &self,
        _ctx: &bijux_cli_core::kernel::ExecutionContext,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<serde_json::Value, bijux_cli_contracts::ErrorEnvelopeV1>,
                > + Send
                + '_,
        >,
    > {
        Box::pin(future::ready(Ok(json!({"ok": "async"}))))
    }
}

struct NoopDiag;
impl DiagnosticsHook for NoopDiag {
    fn record(&self, _record: bijux_cli_contracts::DiagnosticRecord) {}
}

struct LifecycleCounter {
    plugins: Arc<AtomicBool>,
    repl_start: Arc<AtomicBool>,
    repl_shutdown: Arc<AtomicBool>,
}

impl LifecycleHook for LifecycleCounter {
    fn on_plugin_load(&self) {
        self.plugins.store(true, Ordering::SeqCst);
    }

    fn on_repl_start(&self) {
        self.repl_start.store(true, Ordering::SeqCst);
    }

    fn on_repl_shutdown(&self) {
        self.repl_shutdown.store(true, Ordering::SeqCst);
    }
}

fn defaults() -> GlobalFlags {
    GlobalFlags {
        output_format: Some(OutputFormat::Json),
        pretty_mode: Some(PrettyMode::Pretty),
        color_mode: Some(ColorMode::Auto),
        log_level: Some(LogLevel::Info),
        quiet: false,
        include_runtime: false,
    }
}

#[test]
fn resolves_policy_with_expected_precedence() {
    let intent = ExecutionIntent {
        command_path: vec!["cli".to_string(), "status".to_string()],
        global_flags: GlobalFlags {
            output_format: Some(OutputFormat::Yaml),
            pretty_mode: None,
            color_mode: None,
            log_level: Some(LogLevel::Debug),
            quiet: false,
            include_runtime: true,
        },
        args: vec![],
    };

    let policy = resolve_policy(
        &intent,
        &PolicyInputs {
            env: GlobalFlags {
                output_format: Some(OutputFormat::Json),
                pretty_mode: Some(PrettyMode::Compact),
                color_mode: Some(ColorMode::Never),
                log_level: Some(LogLevel::Warning),
                quiet: false,
                include_runtime: false,
            },
            config: defaults(),
            defaults: defaults(),
        },
    );

    assert_eq!(policy.output_format, OutputFormat::Yaml);
    assert_eq!(policy.log_level, LogLevel::Debug);
    assert_eq!(policy.pretty_mode, PrettyMode::Compact);
    assert_eq!(policy.color_mode, ColorMode::Never);
    assert!(policy.include_runtime);
}

#[test]
fn pipeline_runs_sync_and_async_handlers() {
    let intent =
        build_intent_from_argv(&["bijux".to_string(), "cli".to_string(), "status".to_string()]);
    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };
    let cancelled = Arc::new(AtomicBool::new(false));
    let ctx = assemble_context(intent, policy, Some(Duration::from_secs(5)), cancelled, true);

    let diagnostics: Vec<Arc<dyn DiagnosticsHook>> = vec![Arc::new(NoopDiag)];
    let lifecycle: Vec<Arc<dyn LifecycleHook>> = Vec::new();

    let sync_result =
        execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncOk)), &diagnostics, &lifecycle)
            .expect("sync should execute");
    assert_eq!(sync_result.exit_code, ExitCode::Success);
    assert!(sync_result.emission.is_some());
    assert!(sync_result.trace.is_some());

    let async_result =
        execute_pipeline(&ctx, &Handler::Async(Box::new(AsyncOk)), &diagnostics, &lifecycle)
            .expect("async should execute");
    assert_eq!(async_result.exit_code, ExitCode::Success);
}

#[test]
fn pipeline_handles_fast_paths_and_cancellation() {
    let intent = build_intent_from_argv(&["bijux".to_string(), "help".to_string()]);
    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };

    let cancelled = Arc::new(AtomicBool::new(false));
    let ctx =
        assemble_context(intent, policy, Some(Duration::from_secs(5)), cancelled.clone(), true);

    let result = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncOk)), &[], &[])
        .expect("fast-path should succeed");
    assert_eq!(result.exit_code, ExitCode::Success);

    cancelled.store(true, Ordering::SeqCst);
    let cancelled_result = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncOk)), &[], &[])
        .expect_err("cancelled run should fail");
    assert_eq!(cancelled_result, KernelError::Cancelled);
}

#[test]
fn pipeline_invokes_plugin_and_repl_lifecycle_hooks() {
    let plugin_flag = Arc::new(AtomicBool::new(false));
    let repl_start_flag = Arc::new(AtomicBool::new(false));
    let repl_shutdown_flag = Arc::new(AtomicBool::new(false));

    let hook = Arc::new(LifecycleCounter {
        plugins: plugin_flag.clone(),
        repl_start: repl_start_flag.clone(),
        repl_shutdown: repl_shutdown_flag.clone(),
    });

    let lifecycle: Vec<Arc<dyn LifecycleHook>> = vec![hook];

    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };

    let plugin_ctx = assemble_context(
        build_intent_from_argv(&["bijux".to_string(), "plugins".to_string(), "list".to_string()]),
        policy.clone(),
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );
    let _ = execute_pipeline(&plugin_ctx, &Handler::Sync(Box::new(SyncOk)), &[], &lifecycle)
        .expect("plugin path should execute");
    assert!(plugin_flag.load(Ordering::SeqCst));

    let repl_ctx = assemble_context(
        build_intent_from_argv(&["bijux".to_string(), "repl".to_string()]),
        policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );
    let _ = execute_pipeline(&repl_ctx, &Handler::Sync(Box::new(SyncOk)), &[], &lifecycle)
        .expect("repl path should execute");
    assert!(repl_start_flag.load(Ordering::SeqCst));
    assert!(repl_shutdown_flag.load(Ordering::SeqCst));
}

#[test]
fn maps_usage_category_to_stable_usage_exit_code() {
    assert_eq!(map_error_category_to_exit("usage"), ExitCode::Usage);
    assert_eq!(map_error_category_to_exit("validation"), ExitCode::Usage);
}

#[test]
fn maps_validation_plugin_and_internal_categories_to_stable_exit_codes() {
    assert_eq!(map_error_category_to_exit("validation"), ExitCode::Usage);
    assert_eq!(map_error_category_to_exit("plugin"), ExitCode::Error);
    assert_eq!(map_error_category_to_exit("internal"), ExitCode::Error);
}
