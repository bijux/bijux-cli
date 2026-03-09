#![forbid(unsafe_code)]
//! Direct core invocation and parity-oriented policy tests.

use anyhow as _;
use bijux_cli_contracts::{
    ColorMode, ExecutionPolicy, GlobalFlags, LogLevel, OutputFormat, PrettyMode,
};
use bijux_cli_core::app::run_app;
use bijux_cli_core::kernel::{
    build_intent_from_argv, internal_error, map_error_category_to_exit, resolve_policy, usage_error,
    ExecutionContext, ExecutionIntent, HandlerOutcome, PolicyInputs,
};
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_plugin as _;
use bijux_cli_routing as _;
use clap as _;
use futures as _;
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

fn baseline_flags() -> GlobalFlags {
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
fn direct_core_invocation_version() {
    let out = run_app(&["bijux".to_string(), "version".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload.get("version").is_some());
}

#[test]
fn direct_core_invocation_doctor() {
    let out = run_app(&["bijux".to_string(), "doctor".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "healthy");
    assert!(payload["install"]["has_path_shadowing"].is_boolean());
    assert!(payload["install"]["has_duplicate_installs"].is_boolean());
    assert!(payload["install"]["stale_wrapper_scripts"].is_array());
    assert!(payload["install"]["legacy_installer_conflicts"].is_boolean());
}

#[test]
fn direct_core_invocation_status() {
    let out = run_app(&["bijux".to_string(), "status".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn direct_core_invocation_cli_status() {
    let out =
        run_app(&["bijux".to_string(), "cli".to_string(), "status".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn direct_core_invocation_cli_paths() {
    let out = run_app(&["bijux".to_string(), "cli".to_string(), "paths".to_string()])
        .expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload.get("config").is_some());
    assert!(payload.get("history").is_some());
    assert!(payload.get("plugins").is_some());
    assert!(payload.get("path_binaries").is_some());
    assert!(payload.get("post_install_hint").is_some());
}

#[test]
fn direct_core_invocation_inspect() {
    let out = run_app(&["bijux".to_string(), "inspect".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload.get("reserved_namespaces").is_some());
    assert!(payload.get("builtins").is_some());
}

#[test]
fn precedence_flags_over_env() {
    let intent = ExecutionIntent {
        command_path: vec!["cli".to_string(), "status".to_string()],
        global_flags: GlobalFlags {
            output_format: Some(OutputFormat::Yaml),
            pretty_mode: Some(PrettyMode::Compact),
            color_mode: Some(ColorMode::Never),
            log_level: Some(LogLevel::Debug),
            quiet: false,
            include_runtime: false,
        },
        args: vec![],
    };

    let policy = resolve_policy(
        &intent,
        &PolicyInputs {
            env: GlobalFlags {
                output_format: Some(OutputFormat::Json),
                pretty_mode: Some(PrettyMode::Pretty),
                color_mode: Some(ColorMode::Always),
                log_level: Some(LogLevel::Warning),
                quiet: false,
                include_runtime: true,
            },
            config: baseline_flags(),
            defaults: baseline_flags(),
        },
    );

    assert_eq!(policy.output_format, OutputFormat::Yaml);
    assert_eq!(policy.pretty_mode, PrettyMode::Compact);
    assert_eq!(policy.color_mode, ColorMode::Never);
    assert_eq!(policy.log_level, LogLevel::Debug);
}

#[test]
fn precedence_env_over_config() {
    let intent = ExecutionIntent {
        command_path: vec!["cli".to_string(), "status".to_string()],
        global_flags: GlobalFlags::empty(),
        args: vec![],
    };

    let policy = resolve_policy(
        &intent,
        &PolicyInputs {
            env: GlobalFlags {
                output_format: Some(OutputFormat::Yaml),
                pretty_mode: Some(PrettyMode::Compact),
                color_mode: Some(ColorMode::Never),
                log_level: Some(LogLevel::Debug),
                quiet: false,
                include_runtime: false,
            },
            config: baseline_flags(),
            defaults: baseline_flags(),
        },
    );

    assert_eq!(policy.output_format, OutputFormat::Yaml);
    assert_eq!(policy.pretty_mode, PrettyMode::Compact);
    assert_eq!(policy.color_mode, ColorMode::Never);
    assert_eq!(policy.log_level, LogLevel::Debug);
}

#[test]
fn precedence_defaults_when_no_inputs() {
    let intent = ExecutionIntent {
        command_path: vec!["cli".to_string(), "status".to_string()],
        global_flags: GlobalFlags::empty(),
        args: vec![],
    };

    let defaults = baseline_flags();
    let policy = resolve_policy(
        &intent,
        &PolicyInputs { env: GlobalFlags::empty(), config: GlobalFlags::empty(), defaults: defaults.clone() },
    );

    assert_eq!(policy.output_format, defaults.output_format.expect("defaults format"));
    assert_eq!(policy.pretty_mode, defaults.pretty_mode.expect("defaults pretty"));
    assert_eq!(policy.color_mode, defaults.color_mode.expect("defaults color"));
    assert_eq!(policy.log_level, defaults.log_level.expect("defaults log"));
}

#[test]
fn deterministic_mode_behavior_for_same_input() {
    let first = run_app(&["bijux".to_string(), "cli".to_string(), "status".to_string()])
        .expect("first run should succeed");
    let second = run_app(&["bijux".to_string(), "cli".to_string(), "status".to_string()])
        .expect("second run should succeed");

    assert_eq!(first.exit_code, second.exit_code);
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn trace_mode_does_not_change_functional_results() {
    let traced = run_app(&[
        "bijux".to_string(),
        "--log-level".to_string(),
        "trace".to_string(),
        "cli".to_string(),
        "status".to_string(),
    ])
    .expect("traced run should succeed");

    let plain = run_app(&["bijux".to_string(), "cli".to_string(), "status".to_string()])
        .expect("plain run should succeed");

    let traced_value: Value = serde_json::from_str(&traced.stdout).expect("trace output should be json");
    let plain_value: Value = serde_json::from_str(&plain.stdout).expect("plain output should be json");
    assert_eq!(traced.exit_code, plain.exit_code);
    assert_eq!(traced_value["status"], plain_value["status"]);
}

#[test]
fn quiet_mode_suppresses_streams_not_exit_semantics() {
    let quiet = run_app(&[
        "bijux".to_string(),
        "--quiet".to_string(),
        "cli".to_string(),
        "status".to_string(),
    ])
    .expect("quiet run should succeed");

    let plain = run_app(&["bijux".to_string(), "cli".to_string(), "status".to_string()])
        .expect("plain run should succeed");

    assert_eq!(quiet.exit_code, plain.exit_code);
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());
}

#[test]
fn error_normalization_internal_plugin_usage_validation() {
    assert_eq!(map_error_category_to_exit("internal"), bijux_cli_contracts::ExitCode::Error);
    assert_eq!(map_error_category_to_exit("plugin"), bijux_cli_contracts::ExitCode::Error);
    assert_eq!(map_error_category_to_exit("usage"), bijux_cli_contracts::ExitCode::Usage);
    assert_eq!(map_error_category_to_exit("validation"), bijux_cli_contracts::ExitCode::Usage);
}

#[test]
fn usage_and_internal_outcomes_carry_expected_categories() {
    let ctx = ExecutionContext {
        intent: build_intent_from_argv(&["bijux".to_string(), "status".to_string()]),
        policy: ExecutionPolicy::baseline(),
        timeout: Some(Duration::from_secs(1)),
        cancelled: Arc::new(AtomicBool::new(false)),
        trace_mode: false,
    };

    let usage = usage_error(&ctx, "bad usage");
    let internal = internal_error(&ctx, "internal fault");

    match usage {
        HandlerOutcome::Error(error) => assert_eq!(error.error.category, "usage"),
        HandlerOutcome::Success(_) => panic!("usage should emit error"),
    }

    match internal {
        HandlerOutcome::Error(error) => assert_eq!(error.error.category, "internal"),
        HandlerOutcome::Success(_) => panic!("internal should emit error"),
    }
}
