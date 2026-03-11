#![forbid(unsafe_code)]
//! Contracts for routes/registry/env/contracts low-level truth commands.

use std::fs;
use std::process::Command;

use serde_json::Value;

fn run(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.output().expect("binary should execute")
}

fn run_ok_json(command: &[&str], envs: &[(&str, String)]) -> Value {
    let mut args = command.to_vec();
    args.push("--format");
    args.push("json");
    args.push("--no-pretty");
    let out = run(&args, envs);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("json payload")
}

#[test]
fn routes_registry_env_contracts_json_and_text_contracts() {
    let routes = run_ok_json(&["dev", "cli", "routes"], &[]);
    let registry = run_ok_json(&["dev", "cli", "registry"], &[]);
    let env = run_ok_json(&["dev", "cli", "env"], &[]);
    let contracts = run_ok_json(&["dev", "cli", "contracts"], &[]);
    assert!(routes["routes"].is_array());
    assert!(registry["registry"].is_array());
    assert!(env["source_precedence"].is_array());
    assert!(contracts["contracts"].is_object() || contracts["contracts"].is_array());

    for command in [
        ["dev", "cli", "routes"],
        ["dev", "cli", "registry"],
        ["dev", "cli", "env"],
        ["dev", "cli", "contracts"],
    ] {
        let out = run(&[command[0], command[1], command[2], "--format", "text"], &[]);
        assert!(out.status.success(), "text command failed for {:?}", command);
        assert!(!String::from_utf8_lossy(&out.stdout).trim().is_empty());
    }
}

#[test]
fn routes_agrees_with_inspect_command_tree_roots() {
    let routes = run_ok_json(&["dev", "cli", "routes"], &[]);
    let inspect = run_ok_json(&["inspect"], &[]);
    let route_roots: std::collections::BTreeSet<String> = routes["routes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("segments"))
        .filter_map(Value::as_array)
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();
    let inspect_roots: std::collections::BTreeSet<String> = inspect["route_sources"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("segments"))
        .filter_map(Value::as_array)
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();
    assert!(route_roots.iter().all(|root| inspect_roots.contains(root)));
}

#[test]
fn registry_deterministic_with_broken_and_healthy_plugin_files() {
    let root = std::env::temp_dir().join(format!("bijux-registry-mix-{}", std::process::id()));
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir");
    fs::write(plugins.join("healthy.toml"), "[plugin]\nname='healthy'\nentry='plugin:main'\n")
        .expect("write healthy");
    fs::write(plugins.join("broken.toml"), "not a manifest").expect("write broken");
    let envs = [("BIJUX_PLUGINS_DIR", plugins.to_string_lossy().to_string())];
    let first = run_ok_json(&["dev", "cli", "registry"], &envs);
    let second = run_ok_json(&["dev", "cli", "registry"], &envs);
    assert_eq!(first, second, "registry output drift with mixed plugin health");
}

#[test]
fn env_ignores_unrelated_environment_noise() {
    let base = run_ok_json(&["dev", "cli", "env"], &[]);
    let noisy = run_ok_json(
        &["dev", "cli", "env"],
        &[("UNRELATED_NOISE_KEY", "1".to_string()), ("UNRELATED_NOISE_TRACE", "loud".to_string())],
    );
    assert_eq!(base["source_precedence"], noisy["source_precedence"]);
}

#[test]
fn contracts_reports_schema_metadata_and_is_deterministic() {
    let first = run_ok_json(&["dev", "cli", "contracts"], &[]);
    let second = run_ok_json(&["dev", "cli", "contracts"], &[]);
    assert!(first["schema_version"].is_string());
    assert!(first["runtime_version"].is_string());
    assert_eq!(first, second, "contracts output drift");
}

#[test]
fn contracts_all_report_exposes_nextest_style_summary() {
    let report = run_ok_json(&["dev", "cli", "contracts", "--all", "--kind", "status"], &[]);
    assert_eq!(report["kind"], "dev_cli_contracts_all_report_v1");
    assert_eq!(report["mode"], "all");
    assert!(report["contracts"].is_array());
    assert!(report["inventory"]["kinds"].is_object());
    assert!(report["summary"]["nextest_style"]
        .as_str()
        .unwrap_or_default()
        .contains("Summary [contracts]"));
}

#[test]
fn routes_deterministic_when_plugin_file_order_changes() {
    let root = std::env::temp_dir().join(format!("bijux-routes-order-{}", std::process::id()));
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir");
    fs::write(plugins.join("zeta.toml"), "[plugin]\nname='zeta'\nentry='plugin:main'\n")
        .expect("write zeta");
    fs::write(plugins.join("alpha.toml"), "[plugin]\nname='alpha'\nentry='plugin:main'\n")
        .expect("write alpha");
    let envs = [("BIJUX_PLUGINS_DIR", plugins.to_string_lossy().to_string())];
    let first = run_ok_json(&["dev", "cli", "routes"], &envs);
    let second = run_ok_json(&["dev", "cli", "routes"], &envs);
    assert_eq!(first, second, "routes output drift with plugin ordering");
}
