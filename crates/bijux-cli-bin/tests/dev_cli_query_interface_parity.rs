#![forbid(unsafe_code)]
//! Ensures dev-cli reports remain consistent with runtime query interfaces.

use std::process::Command;

use bijux_cli_core::query::state_diagnostics_query;
use bijux_cli_core::install::query::runtime_identity_query;
use bijux_cli_routing::inventory::{registry_inventory, route_inventory};
use bijux_cli_routing::query::contracts_schema_query;
use bijux_cli_routing::registry::RouteRegistry;
use serde_json::Value;

fn run_ok_json(args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute");
    assert!(out.status.success(), "command failed for {args:?}");
    serde_json::from_slice(&out.stdout).expect("valid json")
}

fn run_ok_json_with_env(args: &[&str], envs: &[(&str, String)]) -> Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    let out = command.output().expect("binary should execute");
    assert!(out.status.success(), "command failed for {args:?}");
    serde_json::from_slice(&out.stdout).expect("valid json")
}

#[test]
fn routes_and_registry_reports_match_runtime_query_surfaces() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("community").expect("register");
    let route_query = route_inventory(&registry);
    let registry_query = registry_inventory(&registry);

    let routes = run_ok_json(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]);
    let registry_report =
        run_ok_json(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]);

    assert_eq!(
        routes["routes"].as_array().map_or(0, Vec::len),
        route_query.routes.len(),
        "dev cli routes should use route inventory query rows"
    );
    assert_eq!(
        registry_report["registry"].as_array().map_or(0, Vec::len),
        registry_query.namespaces.len(),
        "dev cli registry should use registry inventory query rows"
    );
}

#[test]
fn runtime_identity_and_state_audit_match_runtime_query_surfaces() {
    let state_root =
        std::env::temp_dir().join(format!("bijux-dev-cli-query-parity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&state_root);
    std::fs::create_dir_all(&state_root).expect("mkdir");

    let config = state_root.join("config.env");
    let history = state_root.join("history.json");
    let memory = state_root.join("memory.json");
    std::fs::write(&config, "A=1\n").expect("write config");

    let plugins_dir = state_root.join("plugins");
    std::fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let envs = vec![
        ("BIJUXCLI_CONFIG", config.to_string_lossy().to_string()),
        ("BIJUXCLI_HISTORY_FILE", history.to_string_lossy().to_string()),
        ("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_string_lossy().to_string()),
    ];
    let state_query =
        state_diagnostics_query(&config, &history, &plugins_dir.join("registry.json"), &memory);
    let state_audit = run_ok_json_with_env(
        &["dev", "cli", "state-audit", "--format", "json", "--no-pretty"],
        &envs,
    );
    assert_eq!(state_audit["paths"]["config"]["exists"], state_query.config.exists);
    assert_eq!(state_audit["paths"]["memory"]["exists"], state_query.memory.exists);

    let identity_query = runtime_identity_query(
        &std::env::var("PATH").unwrap_or_default(),
        std::env::var("BIJUX_BIN").ok().as_deref(),
        std::env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
        env!("CARGO_PKG_VERSION"),
    );
    let runtime_identity =
        run_ok_json(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"]);
    assert_eq!(
        runtime_identity["active_binary"].as_str().map(ToString::to_string),
        identity_query.active_binary
    );
    assert_eq!(
        runtime_identity["path_binaries"].as_array().map_or(0, Vec::len),
        identity_query.path_binaries.len()
    );

    let _ = std::fs::remove_dir_all(&state_root);
}

#[test]
fn contracts_report_matches_contracts_schema_query_surface() {
    let query = contracts_schema_query();
    let report = run_ok_json(&["dev", "cli", "contracts", "--format", "json", "--no-pretty"]);
    assert_eq!(report["schema_version"], query.schema_version);
    assert_eq!(report["contracts"].as_array().map_or(0, Vec::len), query.schema_ids.len());
}
