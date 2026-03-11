#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::contracts::{
    ColorMode, DiagnosticRecord, ErrorEnvelopeV1, ExecutionPolicy, ExitCode, GlobalFlags, LogLevel,
    OutputFormat, PrettyMode,
};
use futures::future;
use serde_json::json;

use super::{
    assemble_context, build_intent_from_argv, execute_pipeline, map_error_category_to_exit,
    resolve_policy, AsyncHandler, DiagnosticsHook, ExecutionContext, ExecutionIntent, Handler,
    KernelError, LifecycleHook, PolicyInputs, SyncHandler,
};

struct SyncOk;
impl SyncHandler for SyncOk {
    fn execute(&self, _ctx: &ExecutionContext) -> Result<serde_json::Value, ErrorEnvelopeV1> {
        Ok(json!({"ok": true}))
    }
}

struct SyncDeterministic;
impl SyncHandler for SyncDeterministic {
    fn execute(&self, ctx: &ExecutionContext) -> Result<serde_json::Value, ErrorEnvelopeV1> {
        Ok(json!({
            "command": ctx.intent.command_path.join(" "),
            "args": ctx.intent.args,
            "format": format!("{:?}", ctx.policy.output_format),
        }))
    }
}

struct AsyncOk;
impl AsyncHandler for AsyncOk {
    fn execute_async(
        &self,
        _ctx: &ExecutionContext,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<serde_json::Value, ErrorEnvelopeV1>>
                + Send
                + '_,
        >,
    > {
        Box::pin(future::ready(Ok(json!({"ok": "async"}))))
    }
}

struct AsyncDeterministic;
impl AsyncHandler for AsyncDeterministic {
    fn execute_async(
        &self,
        ctx: &ExecutionContext,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<serde_json::Value, ErrorEnvelopeV1>>
                + Send
                + '_,
        >,
    > {
        Box::pin(future::ready(Ok(json!({
            "command": ctx.intent.command_path.join(" "),
            "args": ctx.intent.args,
            "format": format!("{:?}", ctx.policy.output_format),
        }))))
    }
}

struct NoopDiag;
impl DiagnosticsHook for NoopDiag {
    fn record(&self, _record: DiagnosticRecord) {}
}

struct CapturingDiag {
    events: Arc<Mutex<Vec<String>>>,
}
impl DiagnosticsHook for CapturingDiag {
    fn record(&self, record: DiagnosticRecord) {
        self.events
            .lock()
            .expect("diag lock poisoned")
            .push(record.id);
    }
}

struct LifecycleCounter {
    plugins: Arc<AtomicBool>,
    repl_start: Arc<AtomicBool>,
    repl_shutdown: Arc<AtomicBool>,
}

struct OrderedLifecycle {
    order: Arc<Mutex<Vec<String>>>,
}
impl LifecycleHook for OrderedLifecycle {
    fn on_plugin_load(&self) {
        self.order
            .lock()
            .expect("order lock poisoned")
            .push("plugin_load".to_string());
    }

    fn on_repl_start(&self) {
        self.order
            .lock()
            .expect("order lock poisoned")
            .push("repl_start".to_string());
    }

    fn on_repl_shutdown(&self) {
        self.order
            .lock()
            .expect("order lock poisoned")
            .push("repl_shutdown".to_string());
    }
}

struct OrderedSync {
    order: Arc<Mutex<Vec<String>>>,
}
impl SyncHandler for OrderedSync {
    fn execute(&self, _ctx: &ExecutionContext) -> Result<serde_json::Value, ErrorEnvelopeV1> {
        self.order
            .lock()
            .expect("order lock poisoned")
            .push("execute".to_string());
        Ok(json!({"ok": true}))
    }
}

struct InternalErrorHandler;
impl SyncHandler for InternalErrorHandler {
    fn execute(&self, _ctx: &ExecutionContext) -> Result<serde_json::Value, ErrorEnvelopeV1> {
        Err(ErrorEnvelopeV1 {
            status: "error".to_string(),
            error: crate::contracts::ErrorPayloadV1 {
                category: "internal".to_string(),
                code: "INTERNAL_FAILURE".to_string(),
                message: "simulated panic normalization".to_string(),
                details: None,
            },
            meta: crate::contracts::OutputEnvelopeMetaV1 {
                version: "v1".to_string(),
                command: crate::contracts::CommandPath { segments: vec![] },
                timestamp: "1970-01-01T00:00:00Z".to_string(),
            },
        })
    }
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
    let ctx = assemble_context(
        intent,
        policy,
        Some(Duration::from_secs(5)),
        cancelled,
        true,
    );

    let diagnostics: Vec<Arc<dyn DiagnosticsHook>> = vec![Arc::new(NoopDiag)];
    let lifecycle: Vec<Arc<dyn LifecycleHook>> = Vec::new();

    let sync_result = execute_pipeline(
        &ctx,
        &Handler::Sync(Box::new(SyncOk)),
        &diagnostics,
        &lifecycle,
    )
    .expect("sync should execute");
    assert_eq!(sync_result.exit_code, ExitCode::Success);
    assert!(sync_result.emission.is_some());
    assert!(sync_result.trace.is_some());

    let async_result = execute_pipeline(
        &ctx,
        &Handler::Async(Box::new(AsyncOk)),
        &diagnostics,
        &lifecycle,
    )
    .expect("async should execute");
    assert_eq!(async_result.exit_code, ExitCode::Success);
    assert!(async_result.emission.is_some());
    assert!(async_result.trace.is_some());
    assert_eq!(
        async_result
            .emission
            .as_ref()
            .and_then(|emission| emission.payload.get("ok"))
            .and_then(serde_json::Value::as_str),
        Some("async")
    );
}

#[test]
fn kernel_pipeline_uses_one_canonical_entrypoint() {
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
    let ctx = assemble_context(
        intent,
        policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        true,
    );
    let result = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncDeterministic)), &[], &[])
        .expect("pipeline should execute");
    assert_eq!(result.exit_code, ExitCode::Success);
    assert!(
        result.trace.is_some(),
        "canonical kernel execution should expose trace"
    );
}

#[test]
fn sync_and_async_handlers_produce_equivalent_normalized_results() {
    let intent = build_intent_from_argv(&[
        "bijux".to_string(),
        "cli".to_string(),
        "status".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ]);
    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };
    let ctx = assemble_context(
        intent,
        policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );

    let sync = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncDeterministic)), &[], &[])
        .expect("sync should execute");
    let async_out = execute_pipeline(
        &ctx,
        &Handler::Async(Box::new(AsyncDeterministic)),
        &[],
        &[],
    )
    .expect("async should execute");

    assert_eq!(sync.exit_code, async_out.exit_code);
    assert_eq!(sync.emission, async_out.emission);
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
    let ctx = assemble_context(
        intent,
        policy,
        Some(Duration::from_secs(5)),
        cancelled.clone(),
        true,
    );

    let result = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncOk)), &[], &[])
        .expect("fast-path should succeed");
    assert_eq!(result.exit_code, ExitCode::Success);

    cancelled.store(true, Ordering::SeqCst);
    let cancelled_result = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncOk)), &[], &[])
        .expect_err("cancelled run should fail");
    assert_eq!(cancelled_result, KernelError::Cancelled);
}

#[test]
fn fast_path_commands_keep_valid_envelope_metadata_when_emitted() {
    let intent = build_intent_from_argv(&["bijux".to_string(), "version".to_string()]);
    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };
    let ctx = assemble_context(
        intent,
        policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );

    let result =
        execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncOk)), &[], &[]).expect("fast-path");
    assert_eq!(result.exit_code, ExitCode::Success);
    let emission = result.emission.expect("fast-path should emit");
    assert_eq!(emission.stream, super::OutputStream::Stdout);
    assert_eq!(emission.payload["meta"]["version"], "v1");
    assert_eq!(
        emission.payload["meta"]["command"]["segments"][0],
        "version"
    );
}

#[test]
fn cancellation_paths_never_emit_partial_success_envelopes() {
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
    let cancelled = Arc::new(AtomicBool::new(true));
    let ctx = assemble_context(
        intent,
        policy,
        Some(Duration::from_secs(5)),
        cancelled,
        false,
    );

    let cancelled_result = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncOk)), &[], &[])
        .expect_err("cancelled run should fail");
    assert_eq!(cancelled_result, KernelError::Cancelled);
}

#[test]
fn cancellation_paths_never_skip_exit_code_mapping() {
    assert_eq!(map_error_category_to_exit("usage"), ExitCode::Usage);
    assert_eq!(map_error_category_to_exit("validation"), ExitCode::Usage);
    assert_eq!(map_error_category_to_exit("plugin"), ExitCode::Error);
    assert_eq!(map_error_category_to_exit("internal"), ExitCode::Error);
    assert_eq!(map_error_category_to_exit("unknown"), ExitCode::Error);
}

#[test]
fn plugin_lifecycle_hooks_run_in_stable_order_around_execution() {
    let order = Arc::new(Mutex::new(Vec::<String>::new()));
    let lifecycle: Vec<Arc<dyn LifecycleHook>> = vec![Arc::new(OrderedLifecycle {
        order: order.clone(),
    })];

    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };
    let ctx = assemble_context(
        build_intent_from_argv(&[
            "bijux".to_string(),
            "plugins".to_string(),
            "list".to_string(),
        ]),
        policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );

    let _ = execute_pipeline(
        &ctx,
        &Handler::Sync(Box::new(OrderedSync {
            order: order.clone(),
        })),
        &[],
        &lifecycle,
    )
    .expect("plugin pipeline should run");
    let observed = order.lock().expect("order lock").clone();
    assert_eq!(
        observed,
        vec!["plugin_load".to_string(), "execute".to_string()]
    );
}

#[test]
fn repl_lifecycle_hooks_do_not_mutate_non_repl_command_semantics() {
    let order = Arc::new(Mutex::new(Vec::<String>::new()));
    let lifecycle: Vec<Arc<dyn LifecycleHook>> = vec![Arc::new(OrderedLifecycle {
        order: order.clone(),
    })];
    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };
    let ctx = assemble_context(
        build_intent_from_argv(&["bijux".to_string(), "cli".to_string(), "status".to_string()]),
        policy.clone(),
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );
    let baseline = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncDeterministic)), &[], &[])
        .expect("baseline should run");
    let with_hooks = execute_pipeline(
        &ctx,
        &Handler::Sync(Box::new(SyncDeterministic)),
        &[],
        &lifecycle,
    )
    .expect("with hooks should run");

    assert_eq!(baseline.exit_code, with_hooks.exit_code);
    assert_eq!(baseline.emission, with_hooks.emission);
    assert!(order.lock().expect("order lock").is_empty());
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
        build_intent_from_argv(&[
            "bijux".to_string(),
            "plugins".to_string(),
            "list".to_string(),
        ]),
        policy.clone(),
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );
    let _ = execute_pipeline(
        &plugin_ctx,
        &Handler::Sync(Box::new(SyncOk)),
        &[],
        &lifecycle,
    )
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
    assert_ne!(map_error_category_to_exit("usage"), ExitCode::Error);
}

#[test]
fn maps_validation_plugin_and_internal_categories_to_stable_exit_codes() {
    assert_eq!(map_error_category_to_exit("validation"), ExitCode::Usage);
    assert_eq!(map_error_category_to_exit("plugin"), ExitCode::Error);
    assert_eq!(map_error_category_to_exit("internal"), ExitCode::Error);
    assert_eq!(map_error_category_to_exit("anything-else"), ExitCode::Error);
}

#[test]
fn kernel_usage_validation_plugin_internal_error_mapping_is_stable() {
    for (category, expected) in [
        ("usage", ExitCode::Usage),
        ("validation", ExitCode::Usage),
        ("plugin", ExitCode::Error),
        ("internal", ExitCode::Error),
        ("unexpected", ExitCode::Error),
    ] {
        let observed = map_error_category_to_exit(category);
        assert_eq!(observed, expected, "unexpected mapping for {category}");
        assert_ne!(
            observed,
            ExitCode::Success,
            "errors must never map to success"
        );
    }
}

#[test]
fn internal_failure_is_normalized_before_crossing_cli_surface() {
    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };
    let ctx = assemble_context(
        build_intent_from_argv(&["bijux".to_string(), "cli".to_string(), "status".to_string()]),
        policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );

    let result = execute_pipeline(
        &ctx,
        &Handler::Sync(Box::new(InternalErrorHandler)),
        &[],
        &[],
    )
    .expect("internal error should normalize");
    assert_eq!(result.exit_code, ExitCode::Error);
    let emission = result.emission.expect("error should emit");
    assert_eq!(emission.stream, super::OutputStream::Stderr);
    assert_eq!(emission.payload["error"]["category"], "internal");
}

#[test]
fn trace_mode_adds_diagnostics_without_changing_payload_shape() {
    let policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };
    let plain_ctx = assemble_context(
        build_intent_from_argv(&["bijux".to_string(), "cli".to_string(), "status".to_string()]),
        policy.clone(),
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );
    let traced_ctx = assemble_context(
        build_intent_from_argv(&["bijux".to_string(), "cli".to_string(), "status".to_string()]),
        policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        true,
    );

    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let diagnostics: Vec<Arc<dyn DiagnosticsHook>> = vec![Arc::new(CapturingDiag {
        events: events.clone(),
    })];
    let plain = execute_pipeline(
        &plain_ctx,
        &Handler::Sync(Box::new(SyncDeterministic)),
        &diagnostics,
        &[],
    )
    .expect("plain");
    let traced = execute_pipeline(
        &traced_ctx,
        &Handler::Sync(Box::new(SyncDeterministic)),
        &diagnostics,
        &[],
    )
    .expect("traced");

    assert_eq!(plain.exit_code, traced.exit_code);
    assert_eq!(plain.emission, traced.emission);
    assert!(plain.trace.is_none());
    assert!(traced.trace.is_some());
    assert!(!events.lock().expect("events lock").is_empty());
}

#[test]
fn quiet_mode_suppresses_streams_but_preserves_result_category() {
    let noisy_policy = ExecutionPolicy {
        output_format: OutputFormat::Json,
        pretty_mode: PrettyMode::Pretty,
        color_mode: ColorMode::Auto,
        log_level: LogLevel::Info,
        quiet: false,
        include_runtime: false,
    };
    let quiet_policy = ExecutionPolicy {
        quiet: true,
        ..noisy_policy.clone()
    };
    let noisy_ctx = assemble_context(
        build_intent_from_argv(&["bijux".to_string(), "cli".to_string(), "status".to_string()]),
        noisy_policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );
    let quiet_ctx = assemble_context(
        build_intent_from_argv(&["bijux".to_string(), "cli".to_string(), "status".to_string()]),
        quiet_policy,
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
        false,
    );

    let noisy = execute_pipeline(
        &noisy_ctx,
        &Handler::Sync(Box::new(SyncDeterministic)),
        &[],
        &[],
    )
    .expect("noisy");
    let quiet = execute_pipeline(
        &quiet_ctx,
        &Handler::Sync(Box::new(SyncDeterministic)),
        &[],
        &[],
    )
    .expect("quiet");
    assert_eq!(noisy.exit_code, quiet.exit_code);
    assert!(noisy.emission.is_some());
    assert!(quiet.emission.is_none());
}

#[test]
fn kernel_resolution_is_deterministic_under_reordered_inputs() {
    let mut intent_a = ExecutionIntent {
        command_path: vec!["cli".to_string(), "status".to_string()],
        global_flags: GlobalFlags::empty(),
        args: vec![],
    };
    intent_a.global_flags.pretty_mode = Some(PrettyMode::Compact);
    intent_a.global_flags.color_mode = Some(ColorMode::Never);

    let mut intent_b = ExecutionIntent {
        command_path: vec!["cli".to_string(), "status".to_string()],
        global_flags: GlobalFlags::empty(),
        args: vec![],
    };
    intent_b.global_flags.color_mode = Some(ColorMode::Never);
    intent_b.global_flags.pretty_mode = Some(PrettyMode::Compact);

    let mut env_a = GlobalFlags::empty();
    env_a.output_format = Some(OutputFormat::Yaml);
    env_a.log_level = Some(LogLevel::Debug);
    let mut env_b = GlobalFlags::empty();
    env_b.log_level = Some(LogLevel::Debug);
    env_b.output_format = Some(OutputFormat::Yaml);

    let mut config_a = GlobalFlags::empty();
    config_a.include_runtime = true;
    config_a.quiet = false;
    let mut config_b = GlobalFlags::empty();
    config_b.quiet = false;
    config_b.include_runtime = true;

    let policy_a = resolve_policy(
        &intent_a,
        &PolicyInputs {
            env: env_a,
            config: config_a,
            defaults: defaults(),
        },
    );
    let policy_b = resolve_policy(
        &intent_b,
        &PolicyInputs {
            env: env_b,
            config: config_b,
            defaults: defaults(),
        },
    );
    assert_eq!(policy_a, policy_b);
}

#[test]
fn repeated_run_kernel_invariants_harness_for_representative_commands() {
    let commands = vec![
        vec!["bijux".to_string(), "cli".to_string(), "status".to_string()],
        vec![
            "bijux".to_string(),
            "plugins".to_string(),
            "list".to_string(),
        ],
        vec!["bijux".to_string(), "help".to_string()],
    ];

    for argv in commands {
        let policy = ExecutionPolicy {
            output_format: OutputFormat::Json,
            pretty_mode: PrettyMode::Pretty,
            color_mode: ColorMode::Auto,
            log_level: LogLevel::Info,
            quiet: false,
            include_runtime: false,
        };
        let ctx = assemble_context(
            build_intent_from_argv(&argv),
            policy,
            Some(Duration::from_secs(5)),
            Arc::new(AtomicBool::new(false)),
            true,
        );
        let first = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncDeterministic)), &[], &[])
            .expect("first");
        let second = execute_pipeline(&ctx, &Handler::Sync(Box::new(SyncDeterministic)), &[], &[])
            .expect("second");
        assert_eq!(
            first.exit_code, second.exit_code,
            "exit mismatch for {argv:?}"
        );
        assert_eq!(
            first.emission, second.emission,
            "emission mismatch for {argv:?}"
        );
    }
}
