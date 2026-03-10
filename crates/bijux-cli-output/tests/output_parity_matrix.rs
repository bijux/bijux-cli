#![forbid(unsafe_code)]
//! Output parity fixtures and regression checks for routed commands.

use std::collections::BTreeMap;
use std::fs;

use bijux_cli_core as _;
use bijux_cli_core::app::run_app;
use bijux_cli_output as _;
use bijux_cli_routing as _;
use serde as _;
use serde::Deserialize;
use serde_json::Value;
use serde_yaml as _;
use thiserror as _;

#[derive(Debug, Deserialize)]
struct CommandFixture {
    required_keys: Vec<String>,
    status_value: Option<String>,
}

fn run(argv: &[&str]) -> (i32, String, String) {
    let input = argv.iter().map(|v| (*v).to_string()).collect::<Vec<_>>();
    let out = run_app(&input).expect("run_app should succeed");
    (out.exit_code, out.stdout, out.stderr)
}

fn fixture_map() -> BTreeMap<String, CommandFixture> {
    let text = fs::read_to_string("tests/fixtures/config_command_fixtures.json")
        .expect("fixture file should load");
    serde_json::from_str(&text).expect("fixture json should parse")
}

#[test]
fn parity_fixtures_for_status_doctor_plugins_cli_status_cli_paths() {
    let fixtures = fixture_map();

    let (_, status_stdout, status_stderr) = run(&["bijux", "cli", "status"]);
    assert!(status_stderr.is_empty());
    let status_json: Value = serde_json::from_str(&status_stdout).expect("status should be json");
    let status_fx = fixtures.get("status").expect("status fixture should exist");
    for key in &status_fx.required_keys {
        assert!(status_json.get(key).is_some(), "missing status key {key}");
    }
    assert_eq!(
        status_json["status"],
        Value::String(status_fx.status_value.clone().expect("status value"))
    );

    let (_, doctor_stdout, doctor_stderr) = run(&["bijux", "doctor"]);
    assert!(doctor_stderr.is_empty());
    let doctor_json: Value = serde_json::from_str(&doctor_stdout).expect("doctor should be json");
    let doctor_fx = fixtures.get("doctor").expect("doctor fixture should exist");
    for key in &doctor_fx.required_keys {
        assert!(doctor_json.get(key).is_some(), "missing doctor key {key}");
    }

    let (_, plugins_stdout, plugins_stderr) = run(&["bijux", "cli", "plugins", "list"]);
    assert!(plugins_stderr.is_empty());
    let plugins_json: Value =
        serde_json::from_str(&plugins_stdout).expect("plugins list should be json");
    let plugins_fx = fixtures.get("plugins_list").expect("plugins fixture should exist");
    for key in &plugins_fx.required_keys {
        assert!(plugins_json.get(key).is_some(), "missing plugins key {key}");
    }

    let (_, alias_status_stdout, alias_status_stderr) = run(&["bijux", "status"]);
    assert!(alias_status_stderr.is_empty());
    let alias_status_json: Value =
        serde_json::from_str(&alias_status_stdout).expect("status alias should be json");
    assert_eq!(alias_status_json["status"], Value::String("ok".to_string()));

    let (_, paths_stdout, paths_stderr) = run(&["bijux", "cli", "paths"]);
    assert!(paths_stderr.is_empty());
    let paths_json: Value = serde_json::from_str(&paths_stdout).expect("cli paths should be json");
    let paths_fx = fixtures.get("cli_paths").expect("paths fixture should exist");
    for key in &paths_fx.required_keys {
        assert!(paths_json.get(key).is_some(), "missing paths key {key}");
    }
}

#[test]
fn compare_rust_outputs_against_python_captures_with_gap_report_guard() {
    let lock = fs::read_to_string("../../artifacts/current-python-behavior-lock.json")
        .expect("python behavior lock should exist");
    let lock_json: Value = serde_json::from_str(&lock).expect("lock should be valid json");
    let captures = lock_json["captures"].as_object().expect("captures should be object");

    let rust_status: Value =
        serde_json::from_str(&run(&["bijux", "status"]).1).expect("status json");
    let py_status: Value = serde_json::from_str(
        captures["bijux_status_json_no_pretty"]["stdout"].as_str().expect("py status stdout"),
    )
    .expect("python status json");
    assert_eq!(rust_status["status"], py_status["status"]);

    let rust_doctor: Value =
        serde_json::from_str(&run(&["bijux", "doctor"]).1).expect("doctor json");
    let py_doctor: Value = serde_json::from_str(
        captures["bijux_doctor"]["stdout"].as_str().expect("py doctor stdout"),
    )
    .expect("python doctor json");
    assert_eq!(rust_doctor["status"], py_doctor["status"]);

    let rust_plugins: Value =
        serde_json::from_str(&run(&["bijux", "cli", "plugins", "list"]).1).expect("plugins json");
    let py_plugins: Value = serde_json::from_str(
        captures["bijux_plugins_list"]["stdout"].as_str().expect("py plugins stdout"),
    )
    .expect("python plugins json");
    assert!(rust_plugins.get("plugins").is_some());
    assert!(py_plugins.get("plugins").is_some());

    let report = fs::read_to_string("../../docs/architecture/output-parity-report.md")
        .expect("output parity report should exist");
    assert!(report.contains("Exact payload parity is not yet complete"));
}

#[test]
fn machine_output_pretty_compact_quiet_and_color_contracts() {
    let (_, compact_stdout, compact_stderr) =
        run(&["bijux", "--format", "json", "--no-pretty", "cli", "status"]);
    assert!(compact_stderr.is_empty());
    assert!(compact_stdout.lines().count() <= 2);

    let (_, pretty_stdout, pretty_stderr) =
        run(&["bijux", "--format", "json", "--pretty", "cli", "status"]);
    assert!(pretty_stderr.is_empty());
    assert!(pretty_stdout.lines().count() > 2);

    let (_, quiet_stdout, quiet_stderr) = run(&["bijux", "--quiet", "cli", "status"]);
    assert!(quiet_stdout.is_empty());
    assert!(quiet_stderr.is_empty());

    let (_, no_color_stdout, no_color_stderr) =
        run(&["bijux", "--color", "never", "--format", "text", "cli", "status"]);
    assert!(no_color_stderr.is_empty());
    assert!(!no_color_stdout.contains("\u{001b}["));
}

#[test]
fn output_size_regression_guard_for_representative_commands() {
    let (_, status_stdout, _) = run(&["bijux", "cli", "status"]);
    let (_, doctor_stdout, _) = run(&["bijux", "doctor"]);
    let (_, paths_stdout, _) = run(&["bijux", "cli", "paths"]);

    assert!(status_stdout.len() <= 512, "status output unexpectedly large");
    assert!(doctor_stdout.len() <= 4096, "doctor output unexpectedly large");
    assert!(paths_stdout.len() <= 4096, "paths output unexpectedly large");
}
