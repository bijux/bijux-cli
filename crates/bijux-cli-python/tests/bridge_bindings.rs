#![forbid(unsafe_code)]
//! Python bridge binding tests for core command parity and error mapping.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow as _;
use bijux_cli_contracts as _;
use bijux_cli_core::app::run_app;
use bijux_cli_install as _;
use bijux_cli_python::{
    classify_failure, cli_status_binding_api, config_resolution_api, doctor_binding_api,
    execution_facade_api, execution_outcome_api, plugins_list_binding_api,
    repl_bootstrap_binding_api, schema_export_helpers_api, status_binding_api, version_binding_api,
    BridgeErrorKind, CompatibilityConfig, PathOverrides, ENV_CONFIG_PATH,
};
use serde_json::Value;
use thiserror as _;

fn parse_json(text: &str) -> Value {
    serde_json::from_str(text).expect("valid json")
}

#[test]
fn version_binding_matches_core_output() {
    let bridge = parse_json(&version_binding_api().expect("bridge version"));
    let direct = parse_json(
        &run_app(&["bijux".to_string(), "version".to_string()]).expect("core run").stdout,
    );
    assert_eq!(bridge, direct);
}

#[test]
fn doctor_binding_matches_core_output() {
    let bridge = parse_json(&doctor_binding_api().expect("bridge doctor"));
    let direct = parse_json(
        &run_app(&["bijux".to_string(), "doctor".to_string()]).expect("core run").stdout,
    );
    assert_eq!(bridge, direct);
}

#[test]
fn status_binding_matches_core_output() {
    let bridge = parse_json(&status_binding_api().expect("bridge status"));
    let direct = parse_json(
        &run_app(&["bijux".to_string(), "status".to_string()]).expect("core run").stdout,
    );
    assert_eq!(bridge, direct);
}

#[test]
fn cli_status_binding_matches_core_output() {
    let bridge = parse_json(&cli_status_binding_api().expect("bridge cli status"));
    let direct = parse_json(
        &run_app(&["bijux".to_string(), "cli".to_string(), "status".to_string()])
            .expect("core run")
            .stdout,
    );
    assert_eq!(bridge, direct);
}

#[test]
fn plugins_list_binding_matches_core_output() {
    let bridge = parse_json(&plugins_list_binding_api().expect("bridge plugins list"));
    let direct = parse_json(
        &run_app(&[
            "bijux".to_string(),
            "cli".to_string(),
            "plugins".to_string(),
            "list".to_string(),
        ])
        .expect("core run")
        .stdout,
    );
    assert_eq!(bridge, direct);
}

#[test]
fn execution_outcome_reports_error_kind_for_unknown_namespace() {
    let payload = parse_json(
        &execution_outcome_api(&["bijux".to_string(), "ghost".to_string(), "status".to_string()])
            .expect("bridge run"),
    );
    assert_eq!(payload["error_kind"], "UsageError");
}

#[test]
fn config_resolution_respects_precedence() {
    let home = PathBuf::from("/tmp/bridge-home");
    let mut env_map = HashMap::new();
    env_map.insert(ENV_CONFIG_PATH.to_string(), "from-env.env".to_string());

    let output = config_resolution_api(
        Some(&home),
        &PathOverrides {
            config_file: Some(PathBuf::from("/cli/config.env")),
            history_file: None,
            plugins_dir: None,
        },
        &env_map,
        &CompatibilityConfig {
            config_file: Some(PathBuf::from("from-config.env")),
            history_file: None,
            plugins_dir: None,
        },
    )
    .expect("resolution");

    let payload = parse_json(&output);
    assert_eq!(payload["config_file"], "/cli/config.env");
}

#[test]
fn plugin_namespace_rejection_returns_usage_error_kind() {
    let payload = parse_json(
        &execution_outcome_api(&[
            "bijux".to_string(),
            "cli".to_string(),
            "plugins".to_string(),
            "unknown-subcommand".to_string(),
        ])
        .expect("bridge run"),
    );
    assert_eq!(payload["error_kind"], "UsageError");
}

#[test]
fn repl_bootstrap_help_is_exposed() {
    let rendered = repl_bootstrap_binding_api().expect("repl help");
    assert!(rendered.contains("Usage:"));
    assert!(rendered.contains("repl"));
}

#[test]
fn schema_export_helpers_are_available() {
    let payload = parse_json(&schema_export_helpers_api());
    assert!(payload["schemas"].is_array());
    assert!(payload["schemas"].as_array().expect("array").len() >= 3);
}

#[test]
fn bridge_execution_matches_direct_core_for_covered_commands() {
    let commands: Vec<Vec<String>> = vec![
        vec!["bijux", "version"],
        vec!["bijux", "doctor"],
        vec!["bijux", "status"],
        vec!["bijux", "cli", "status"],
        vec!["bijux", "cli", "plugins", "list"],
    ]
    .into_iter()
    .map(|line| line.into_iter().map(ToString::to_string).collect())
    .collect();

    for argv in commands {
        let bridge = execution_facade_api(&argv).expect("bridge result");
        let direct = run_app(&argv).expect("core result");
        if direct.exit_code == 0 {
            assert_eq!(bridge, direct.stdout);
        } else if !direct.stderr.is_empty() {
            assert_eq!(bridge, direct.stderr);
        } else {
            assert_eq!(bridge, direct.stdout);
        }
    }
}

#[test]
fn usage_error_mapping_is_stable() {
    let outcome = parse_json(
        &execution_outcome_api(&["bijux".to_string(), "ghost".to_string()]).expect("bridge"),
    );
    assert_eq!(outcome["error_kind"], "UsageError");
    assert_eq!(classify_failure(2, "bad usage"), BridgeErrorKind::Usage);
}

#[test]
fn validation_error_mapping_is_stable() {
    assert_eq!(classify_failure(1, "validation failed for key"), BridgeErrorKind::Validation);
}

#[test]
fn internal_error_mapping_is_stable() {
    assert_eq!(classify_failure(1, "runtime panic path"), BridgeErrorKind::Internal);
}
