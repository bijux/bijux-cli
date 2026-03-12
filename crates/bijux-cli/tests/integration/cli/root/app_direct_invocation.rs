#![forbid(unsafe_code)]
//! Direct core invocation and parity-oriented policy tests.

use anyhow as _;
use bijux_cli::api::kernel::{
    map_error_category_to_exit, resolve_policy, ExecutionIntent, PolicyInputs,
};
use bijux_cli::api::runtime::run_app;
use bijux_cli::contracts::{ColorMode, ExitCode, GlobalFlags, LogLevel, OutputFormat, PrettyMode};
use clap as _;
use futures as _;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn run_success_json(args: &[&str], label: &str) -> Value {
    let argv: Vec<String> = args.iter().map(|value| (*value).to_string()).collect();
    let out = run_app(&argv).expect("run_app should succeed");
    assert_eq!(out.exit_code, 0, "unexpected exit code for {label}");
    assert!(
        out.stderr.is_empty(),
        "stderr should stay empty for success case {label}"
    );
    assert!(
        !out.stdout.trim().is_empty(),
        "stdout should not be empty for success case {label}"
    );
    serde_json::from_str(&out.stdout).expect("valid json")
}

#[test]
fn direct_core_invocation_version() {
    let payload = run_success_json(&["bijux", "version"], "version");
    assert!(payload.get("version").is_some());
}

#[test]
fn direct_core_invocation_doctor() {
    let payload = run_success_json(&["bijux", "doctor"], "doctor");
    assert_eq!(payload["status"], "healthy");
    assert!(payload["install"]["has_path_shadowing"].is_boolean());
    assert!(payload["install"]["has_duplicate_installs"].is_boolean());
    assert!(payload["install"]["stale_wrapper_scripts"].is_array());
    assert!(payload["install"]["legacy_installer_conflicts"].is_boolean());
}

#[test]
fn direct_core_invocation_status() {
    let payload = run_success_json(&["bijux", "status"], "status");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn direct_core_invocation_cli_status() {
    let payload = run_success_json(&["bijux", "cli", "status"], "cli status");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn direct_core_invocation_cli_paths() {
    let payload = run_success_json(&["bijux", "cli", "paths"], "cli paths");
    assert!(payload.get("config").is_some());
    assert!(payload.get("history").is_some());
    assert!(payload.get("plugins").is_some());
    assert!(payload.get("path_binaries").is_some());
    assert!(payload.get("post_install_hint").is_some());
}

#[test]
fn direct_core_invocation_inspect() {
    let payload = run_success_json(&["bijux", "inspect"], "inspect");
    assert_eq!(payload["status"], "ok");
    assert!(payload["reserved_namespaces"].is_array());
    assert!(payload["builtins"].is_array());
    assert!(payload["route_sources"].is_array());
    assert!(payload["alias_rewrites"].is_array());
    assert!(payload["contracts"]["schemas"].is_array());
}

#[test]
fn direct_core_invocation_config_root() {
    let payload = run_success_json(&["bijux", "config"], "config root");
    assert!(payload.is_object());
}

#[test]
fn direct_core_invocation_history_root() {
    let payload = run_success_json(&["bijux", "history"], "history root");
    assert!(payload["entries"].is_array());
}

#[test]
fn direct_core_invocation_memory_root() {
    let payload = run_success_json(&["bijux", "memory"], "memory root");
    assert_eq!(payload["status"], "ok");
    assert!(payload["count"].is_number());
}

#[test]
fn direct_core_invocation_memory_list() {
    let payload = run_success_json(&["bijux", "memory", "list"], "memory list");
    assert_eq!(payload["status"], "ok");
    assert!(payload["keys"].is_array());
}

#[test]
fn direct_core_invocation_plugins_root_list() {
    let payload = run_success_json(&["bijux", "plugins", "list"], "plugins list");
    assert!(payload.get("plugins").is_some());
}

#[test]
fn direct_core_invocation_root_status_without_cli_alias() {
    let payload = run_success_json(&["bijux", "status"], "root status");
    assert_eq!(payload["status"], "ok");
}

#[test]
fn direct_core_invocation_root_audit() {
    let payload = run_success_json(&["bijux", "audit"], "audit");
    assert!(payload["checks"].is_array());
}

#[test]
fn direct_core_invocation_root_docs() {
    let payload = run_success_json(&["bijux", "docs"], "docs");
    assert!(payload["topics"].is_array());
}

#[test]
fn direct_core_invocation_root_sleep() {
    let payload = run_success_json(&["bijux", "sleep", "0"], "sleep");
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
        vec!["bijux", "cli", "plugins", "doctor"],
        vec!["bijux", "dev", "cli", "routes"],
        vec!["bijux", "dev", "cli", "route-audit"],
        vec!["bijux", "dev", "cli", "registry"],
        vec!["bijux", "dev", "cli", "docs-audit"],
        vec!["bijux", "dev", "cli", "maintenance-audit"],
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
        assert!(
            out.stderr.is_empty(),
            "stderr should stay empty for {argv:?}"
        );
        let payload: Value =
            serde_json::from_str(&out.stdout).expect("output should be valid json");
        assert!(
            payload.is_object(),
            "newly ported command should return object envelope for {argv:?}"
        );
    }
}

#[test]
fn direct_core_invocation_dev_diagnostics_commands_expose_metadata() {
    for argv in [
        vec!["bijux", "dev", "cli", "routes"],
        vec!["bijux", "dev", "cli", "route-audit"],
        vec!["bijux", "dev", "cli", "registry"],
        vec!["bijux", "dev", "cli", "docs-audit"],
        vec!["bijux", "dev", "cli", "maintenance-audit"],
        vec!["bijux", "dev", "cli", "env"],
        vec!["bijux", "dev", "cli", "doctor"],
        vec!["bijux", "dev", "cli", "contracts"],
    ] {
        let args: Vec<String> = argv.into_iter().map(ToString::to_string).collect();
        let out = run_app(&args).expect("run_app should succeed");
        assert_eq!(out.exit_code, 0);
        assert!(
            out.stderr.is_empty(),
            "stderr should stay empty for {args:?}"
        );
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
            "route-audit" => {
                assert!(payload["routes"].is_array());
                assert!(payload["aliases"].is_array());
                assert!(payload["summary"].is_object());
            }
            "docs-audit" => {
                assert!(payload["docs_audit"].is_object());
                assert!(payload["docs"].is_array());
                assert!(payload["docs_count"].is_number());
            }
            "maintenance-audit" => {
                assert!(payload["maintenance"].is_array());
                assert!(payload["summary"].is_object());
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

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-state-law-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

#[test]
fn state_audit_reports_all_known_state_files() {
    let out = run_app(&[
        "bijux".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "state-audit".to_string(),
    ])
    .expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    assert!(out.stderr.is_empty());
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert!(payload["paths"]["config"].is_object());
    assert!(payload["paths"]["history"].is_object());
    assert!(payload["paths"]["plugins_registry"].is_object());
    assert!(payload["paths"]["memory"].is_object());
}

#[test]
fn state_read_paths_follow_normalized_resolution_with_flag_overrides() {
    let temp = make_temp_dir("resolved-paths");
    let custom_config = temp.join("custom.env");
    fs::write(&custom_config, "BIJUXCLI_ALPHA=1\n").expect("seed config");

    let out = run_app(&[
        "bijux".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "state-audit".to_string(),
        "--config-path".to_string(),
        custom_config.display().to_string(),
    ])
    .expect("run_app should succeed");

    assert_eq!(out.exit_code, 0);
    assert!(out.stderr.is_empty());
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(
        payload["paths"]["config"]["path"].as_str(),
        Some(custom_config.to_string_lossy().as_ref())
    );
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
    assert!(out.stderr.is_empty());
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
    assert!(payload["reports"]["state_paths_report"].is_object());
    assert!(payload["reports"]["state_corruption_health_report"].is_object());
    assert!(payload["reports"]["state_migration_status"].is_object());
    assert!(payload["reports"]["unified_state_behavior_report"].is_object());
    assert!(payload["reports"]["unified_state_corruption_report"].is_object());
    assert!(payload["reports"]["unified_state_rollback_report"].is_object());
    assert!(payload["reports"]["unified_state_path_resolution_report"].is_object());
    assert!(payload["reports"]["unified_state_doctor_snapshots"].is_object());
    assert!(payload["reports"]["unified_state_audit_payload"].is_object());
    assert!(payload["reports"]["snapshot_coverage"].is_object());
    assert!(payload["reports"]["stream_coverage"].is_object());
    assert!(payload["reports"]["exit_code_coverage"].is_object());
    assert!(payload["reports"]["failure_path_coverage"].is_object());
    assert!(payload["reports"]["compatibility_aliases"].is_object());
    assert!(payload["reports"]["known_parity_gaps"].is_object());
    assert!(payload["reports"]["intentional_differences"].is_object());
    assert!(payload["reports"]["unowned_maintenance"].is_object());
    assert!(payload["reports"]["maintainer_maintenance_outside_dev_cli"].is_object());
    assert!(payload["reports"]["maintainer_control_plane_commands"].is_object());
    assert!(payload["reports"]["maintainer_control_plane_report"].is_object());
    assert!(payload["reports"]["maintainer_control_plane_text_report"].is_string());
    assert!(payload["reports"]["plugin_lifecycle_ownership_report"].is_object());
    assert!(payload["reports"]["plugin_scaffold_efficiency_report"].is_object());
    assert!(payload["reports"]["plugin_scaffold_lifecycle_proof_report"].is_object());
    assert!(payload["reports"]["plugin_namespace_abuse_proof_report"].is_object());
    assert!(payload["reports"]["plugin_doctor_clarity_report"].is_object());
    assert!(payload["reports"]["plugin_explain_clarity_report"].is_object());
    assert!(payload["reports"]["plugin_where_ownership_report"].is_object());
    assert!(payload["reports"]["plugin_command_set_status"].is_object());
    assert!(payload["reports"]["plugin_migration_report"].is_object());
}

#[test]
fn direct_core_invocation_runtime_identity_exposes_runtime_diagnostics() {
    let out = run_app(&[
        "bijux".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "runtime-identity".to_string(),
    ])
    .expect("run_app should succeed");
    assert_eq!(out.exit_code, 0);
    assert!(out.stderr.is_empty());
    let payload: Value = serde_json::from_str(&out.stdout).expect("valid json");
    assert_eq!(payload["canonical_user_binary"], "bijux");
    assert!(payload["public_runtime_binary_names"].is_array());
    assert!(payload["secondary_public_runtime_binary_names"].is_array());
    assert!(payload["active_binary"].is_null() || payload["active_binary"].is_string());
    assert!(payload["install_source"].is_string());
    assert!(payload["active_path_is_canonical_name"].is_boolean());
    assert!(payload["active_path_is_shadowed"].is_boolean());
    assert!(payload["active_binary_selection_is_ambiguous"].is_boolean());
    assert!(payload["diagnostics"]["duplicate_install_detected"].is_boolean());
    assert!(payload["diagnostics"]["mixed_pip_cargo_install_detected"].is_boolean());
    assert!(payload["diagnostics"]["path_shadowing_detected"].is_boolean());
    assert!(payload["diagnostics"]["stale_wrapper_detected"].is_boolean());
    assert!(payload["diagnostics"]["stale_wrapper_maintenance"].is_array());
    assert!(payload["text_summary"].is_array());
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

    assert_eq!(
        policy.output_format,
        defaults.output_format.expect("defaults format")
    );
    assert_eq!(
        policy.pretty_mode,
        defaults.pretty_mode.expect("defaults pretty")
    );
    assert_eq!(
        policy.color_mode,
        defaults.color_mode.expect("defaults color")
    );
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
    let first_payload: Value = serde_json::from_str(&first.stdout).expect("first payload json");
    let second_payload: Value = serde_json::from_str(&second.stdout).expect("second payload json");
    assert_eq!(first_payload, second_payload);
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
    assert_eq!(map_error_category_to_exit("internal"), ExitCode::Error);
    assert_eq!(map_error_category_to_exit("plugin"), ExitCode::Error);
    assert_eq!(map_error_category_to_exit("usage"), ExitCode::Usage);
    assert_eq!(map_error_category_to_exit("validation"), ExitCode::Usage);
}
