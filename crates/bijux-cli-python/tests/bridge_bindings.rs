#![forbid(unsafe_code)]
//! Python bridge binding tests for core command parity and error mapping.

use std::collections::HashMap;
use std::path::PathBuf;

use bijux_cli::interface::cli::dispatch::run_app;
use bijux_cli_python::{
    classify_failure, cli_status_binding_api, command_tree_introspection_api,
    config_resolution_api, doctor_binding_api, execution_facade_api, execution_outcome_api,
    plugins_list_binding_api, repl_bootstrap_binding_api, schema_export_helpers_api,
    status_binding_api, version_binding_api, BridgeErrorKind, CompatibilityConfig, PathOverrides,
    ENV_CONFIG_PATH,
};
use serde_json::Value;

fn parse_json(text: &str) -> Value {
    serde_json::from_str(text).expect("valid json")
}

#[test]
fn version_binding_matches_core_output() {
    let bridge = parse_json(&version_binding_api().expect("bridge version"));
    let direct = parse_json(
        &run_app(&["bijux".to_string(), "version".to_string()])
            .expect("core run")
            .stdout,
    );
    assert_eq!(bridge, direct);
}

#[test]
fn doctor_binding_matches_core_output() {
    let bridge = parse_json(&doctor_binding_api().expect("bridge doctor"));
    let direct = parse_json(
        &run_app(&["bijux".to_string(), "doctor".to_string()])
            .expect("core run")
            .stdout,
    );
    assert_eq!(bridge, direct);
}

#[test]
fn status_binding_matches_core_output() {
    let bridge = parse_json(&status_binding_api().expect("bridge status"));
    let direct = parse_json(
        &run_app(&["bijux".to_string(), "status".to_string()])
            .expect("core run")
            .stdout,
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
        &execution_outcome_api(&[
            "bijux".to_string(),
            "ghost".to_string(),
            "status".to_string(),
        ])
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
    assert_eq!(
        classify_failure(1, "validation failed for key"),
        BridgeErrorKind::Validation
    );
}

#[test]
fn internal_error_mapping_is_stable() {
    assert_eq!(
        classify_failure(1, "runtime panic path"),
        BridgeErrorKind::Internal
    );
}

#[test]
fn binary_and_bridge_use_same_command_registry_contract() {
    let bridge_tree = parse_json(&command_tree_introspection_api());
    let core_inspect = parse_json(
        &run_app(&["bijux".to_string(), "inspect".to_string()])
            .expect("core inspect")
            .stdout,
    );
    let builtins = core_inspect["builtins"].as_array().expect("builtins array");
    let surface: Vec<String> = builtins
        .iter()
        .filter_map(|row| row.get("segments"))
        .filter_map(|segments| segments.as_array())
        .filter_map(|segments| {
            let parts: Vec<&str> = segments.iter().filter_map(Value::as_str).collect();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(" "))
            }
        })
        .collect();

    assert_eq!(bridge_tree["root"], "bijux");
    assert!(
        bridge_tree["namespaces"]
            .as_array()
            .expect("namespaces")
            .len()
            >= 5
    );
    assert!(surface.iter().any(|item| item.starts_with("cli ")));
    assert!(surface.iter().any(|item| item.starts_with("dev ")));
}

#[test]
fn binary_and_bridge_use_same_exit_mapping_for_unknown_route() {
    let argv = vec![
        "bijux".to_string(),
        "ghost".to_string(),
        "status".to_string(),
    ];
    let bridge = parse_json(&execution_outcome_api(&argv).expect("bridge"));
    let core = run_app(&argv).expect("core");
    assert_eq!(
        bridge["exit_code"].as_i64().unwrap_or(-1),
        i64::from(core.exit_code)
    );
}

#[test]
fn binary_and_bridge_use_same_output_envelope_shape() {
    let argv = vec!["bijux".to_string(), "status".to_string()];
    let bridge = parse_json(&execution_facade_api(&argv).expect("bridge"));
    let core = parse_json(&run_app(&argv).expect("core").stdout);
    assert!(bridge.get("status").is_some());
    assert_eq!(bridge.get("status"), core.get("status"));
    assert_eq!(
        bridge.as_object().map(|o| o.len()),
        core.as_object().map(|o| o.len())
    );
}

#[test]
fn binary_and_bridge_use_same_namespace_rejection_logic() {
    let argv = vec![
        "bijux".to_string(),
        "cli".to_string(),
        "plugins".to_string(),
        "unknown-subcommand".to_string(),
    ];
    let bridge = parse_json(&execution_outcome_api(&argv).expect("bridge"));
    let core = run_app(&argv).expect("core");
    assert_eq!(
        bridge["exit_code"].as_i64().unwrap_or(-1),
        i64::from(core.exit_code)
    );
    assert_eq!(bridge["error_kind"], "UsageError");
}

#[test]
fn binary_and_bridge_use_same_plugin_registry_logic_for_listing() {
    let argv = vec![
        "bijux".to_string(),
        "cli".to_string(),
        "plugins".to_string(),
        "list".to_string(),
    ];
    let bridge = parse_json(&execution_facade_api(&argv).expect("bridge"));
    let core = parse_json(&run_app(&argv).expect("core").stdout);
    assert_eq!(bridge, core);
}

#[test]
fn runtime_identity_matches_between_binary_and_bridge() {
    let argv = vec![
        "bijux".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "runtime-identity".to_string(),
    ];
    let bridge = parse_json(&execution_facade_api(&argv).expect("bridge"));
    let core = parse_json(&run_app(&argv).expect("core").stdout);
    assert_eq!(
        bridge["canonical_user_binary"],
        core["canonical_user_binary"]
    );
    assert_eq!(bridge["entrypoints"], core["entrypoints"]);
    assert_eq!(bridge["diagnostics"], core["diagnostics"]);
}

#[test]
fn execution_path_keeps_config_precedence_identical_between_binary_and_bridge() {
    let root = std::env::temp_dir().join(format!(
        "bijux-bridge-config-precedence-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    let config = root.join("config.env");
    std::fs::write(&config, "sample_key=file\n").expect("write config");

    let argv = vec![
        "bijux".to_string(),
        "--config-path".to_string(),
        config.to_string_lossy().to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "sample_key".to_string(),
    ];

    let bridge = parse_json(&execution_outcome_api(&argv).expect("bridge"));
    let core = run_app(&argv).expect("core");
    assert_eq!(
        bridge["exit_code"].as_i64().unwrap_or(-1),
        i64::from(core.exit_code)
    );
    assert_eq!(bridge["stdout"].as_str().unwrap_or_default(), core.stdout);
    assert_eq!(bridge["stderr"].as_str().unwrap_or_default(), core.stderr);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn diagnostics_payloads_match_between_binary_and_bridge() {
    let argv = vec![
        "bijux".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "doctor".to_string(),
    ];
    let bridge = parse_json(&execution_facade_api(&argv).expect("bridge"));
    let core = parse_json(&run_app(&argv).expect("core").stdout);
    assert_eq!(bridge["issues"], core["issues"]);
    assert_eq!(bridge["checks"], core["checks"]);
}
