#![forbid(unsafe_code)]
//! Direct core invocation and parity-oriented policy tests.

use anyhow as _;
use bijux_cli_contracts::{
    ColorMode, ExecutionPolicy, GlobalFlags, LogLevel, OutputFormat, PrettyMode,
};
use bijux_cli_core::app::run_app;
use bijux_cli_core::kernel::{
    build_intent_from_argv, internal_error, map_error_category_to_exit, resolve_policy,
    usage_error, ExecutionContext, ExecutionIntent, HandlerOutcome, PolicyInputs,
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
    let out =
        run_app(&["bijux".to_string(), "version".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload.get("version").is_some());
}

#[test]
fn direct_core_invocation_doctor() {
    let out =
        run_app(&["bijux".to_string(), "doctor".to_string()]).expect("run_app should succeed");
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
    let out =
        run_app(&["bijux".to_string(), "status".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn direct_core_invocation_cli_status() {
    let out = run_app(&["bijux".to_string(), "cli".to_string(), "status".to_string()])
        .expect("run_app should succeed");
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
    let out =
        run_app(&["bijux".to_string(), "inspect".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
    assert!(payload["reserved_namespaces"].is_array());
    assert!(payload["builtins"].is_array());
    assert!(payload["route_sources"].is_array());
    assert!(payload["alias_rewrites"].is_array());
    assert!(payload["contracts"]["schemas"].is_array());
}

#[test]
fn direct_core_invocation_config_root() {
    let out =
        run_app(&["bijux".to_string(), "config".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload.is_object());
}

#[test]
fn direct_core_invocation_history_root() {
    let out =
        run_app(&["bijux".to_string(), "history".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload["entries"].is_array());
}

#[test]
fn direct_core_invocation_memory_root() {
    let out =
        run_app(&["bijux".to_string(), "memory".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
    assert!(payload["count"].is_number());
}

#[test]
fn direct_core_invocation_memory_list() {
    let out = run_app(&["bijux".to_string(), "memory".to_string(), "list".to_string()])
        .expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
    assert!(payload["keys"].is_array());
}

#[test]
fn direct_core_invocation_plugins_root_list() {
    let out = run_app(&["bijux".to_string(), "plugins".to_string(), "list".to_string()])
        .expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload.get("plugins").is_some());
}

#[test]
fn direct_core_invocation_root_status_without_cli_alias() {
    let out =
        run_app(&["bijux".to_string(), "status".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn direct_core_invocation_root_audit() {
    let out = run_app(&["bijux".to_string(), "audit".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload["checks"].is_array());
}

#[test]
fn direct_core_invocation_root_docs() {
    let out = run_app(&["bijux".to_string(), "docs".to_string()]).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload["topics"].is_array());
}

#[test]
fn direct_core_invocation_root_sleep() {
    let out = run_app(&["bijux".to_string(), "sleep".to_string(), "0".to_string()])
        .expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn direct_core_invocation_newly_ported_commands_execute() {
    let cases: Vec<Vec<String>> = vec![
        vec!["bijux", "status"],
        vec!["bijux", "audit"],
        vec!["bijux", "docs"],
        vec!["bijux", "sleep", "0"],
        vec!["bijux", "cli", "config", "set", "TEST_KEY=1"],
        vec!["bijux", "cli", "config", "get", "TEST_KEY"],
        vec!["bijux", "cli", "self-test"],
        vec!["bijux", "cli", "plugins", "list"],
        vec!["bijux", "cli", "plugins", "inspect"],
        vec!["bijux", "dev", "cli", "routes"],
        vec!["bijux", "dev", "cli", "registry"],
        vec!["bijux", "dev", "cli", "env"],
        vec!["bijux", "dev", "cli", "doctor"],
        vec!["bijux", "dev", "cli", "contracts"],
    ]
    .into_iter()
    .map(|parts| parts.into_iter().map(ToString::to_string).collect())
    .collect();

    for argv in cases {
        let out = run_app(&argv).expect("run_app should succeed");
        assert_eq!(out.exit_code, 0, "non-zero exit for {argv:?}");
        let _: Value = serde_json::from_str(&out.stdout).expect("output should be valid json");
    }
}

#[test]
fn direct_core_invocation_dev_diagnostics_commands_expose_metadata() {
    for argv in [
        vec!["bijux", "dev", "cli", "routes"],
        vec!["bijux", "dev", "cli", "registry"],
        vec!["bijux", "dev", "cli", "env"],
        vec!["bijux", "dev", "cli", "doctor"],
        vec!["bijux", "dev", "cli", "contracts"],
    ] {
        let args: Vec<String> = argv.into_iter().map(ToString::to_string).collect();
        let out = run_app(&args).expect("run_app should succeed");
        assert_eq!(out.exit_code, 0);
        let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
        match args[3].as_str() {
            "routes" => {
                assert!(payload["routes"].is_array());
                assert!(payload["aliases"].is_array());
            }
            "registry" => {
                assert!(payload["registry"].is_array());
                assert!(payload["ownership"].is_object());
                assert!(payload["precedence"].is_array());
            }
            "env" => {
                assert!(payload["env"].is_object());
                assert!(payload["source_precedence"].is_array());
                assert!(payload["active"]["config_file"].is_string());
            }
            "doctor" => {
                assert!(payload["issues"]["config"].is_array());
                assert!(payload["issues"]["paths"].is_array());
                assert!(payload["issues"]["plugins"].is_array());
            }
            "contracts" => {
                assert!(payload["contracts"].is_array());
                assert!(payload["schema_version"].is_string());
                assert!(payload["runtime_version"].is_string());
            }
            _ => panic!("unexpected command case"),
        }
    }
}

#[test]
fn direct_core_invocation_dev_status_exposes_generated_report_bundle() {
    let out = run_app(&[
        "bijux".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "status".to_string(),
    ])
    .expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload["status_report"].is_object());
    assert!(payload["reports"]["root_commands"].is_object());
    assert!(payload["reports"]["cli_subcommands"].is_object());
    assert!(payload["reports"]["dev_cli_subcommands"].is_object());
    assert!(payload["reports"]["plugin_commands"].is_object());
    assert!(payload["reports"]["repl_parity_coverage"].is_object());
    assert!(payload["reports"]["python_bridge_parity_coverage"].is_object());
    assert!(payload["reports"]["install_packaging_parity_coverage"].is_object());
    assert!(payload["reports"]["state_behavior_coverage"].is_object());
    assert!(payload["reports"]["snapshot_coverage"].is_object());
    assert!(payload["reports"]["stream_coverage"].is_object());
    assert!(payload["reports"]["exit_code_coverage"].is_object());
    assert!(payload["reports"]["failure_path_coverage"].is_object());
    assert!(payload["reports"]["compatibility_aliases"].is_object());
    assert!(payload["reports"]["known_parity_gaps"].is_object());
    assert!(payload["reports"]["intentional_differences"].is_object());
    assert!(payload["reports"]["unowned_scripts"].is_object());
}

#[test]
fn direct_core_invocation_inspect_failure_normalizes_usage_error() {
    let out = run_app(&[
        "bijux".to_string(),
        "inspect".to_string(),
        "unexpected".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--no-pretty".to_string(),
    ])
    .expect("run_app should return normalized failure");
    assert_eq!(out.exit_code, 2);
    assert!(out.stdout.is_empty());
    assert!(out.stderr.contains("Usage: bijux"));
    assert!(out.stderr.contains("inspect"));
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
        &PolicyInputs {
            env: GlobalFlags::empty(),
            config: GlobalFlags::empty(),
            defaults: defaults.clone(),
        },
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

    let traced_value: Value =
        serde_json::from_str(&traced.stdout).expect("trace output should be json");
    let plain_value: Value =
        serde_json::from_str(&plain.stdout).expect("plain output should be json");
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
