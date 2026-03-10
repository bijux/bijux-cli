#![forbid(unsafe_code)]
//! Deep diagnostics and doctor behavior coverage for TODOs 141-153.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_env(args: &[&str], envs: &[(&str, &Path)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.output().expect("binary should execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json output")
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-diagnostics-deep-{label}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

#[test]
fn doctor_findings_are_stable_and_do_not_reorder_nondeterministically() {
    let first = run(&["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"]);
    let second = run(&["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"]);

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));

    let first_json = parse_json(&first.stdout);
    let second_json = parse_json(&second.stdout);

    assert_eq!(first_json["doctor"]["issues"], second_json["doctor"]["issues"]);
}

#[test]
fn doctor_json_and_text_are_stable_with_no_color_mode() {
    let json_first = run(&["doctor", "--format", "json", "--no-pretty"]);
    let json_second = run(&["doctor", "--format", "json", "--no-pretty"]);
    assert_eq!(json_first.status.code(), Some(0));
    assert_eq!(json_second.status.code(), Some(0));
    assert_eq!(json_first.stdout, json_second.stdout);

    let mut text_cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    text_cmd.args(["doctor", "--format", "text"]);
    text_cmd.env("NO_COLOR", "1");
    let text_first = text_cmd.output().expect("doctor text no color");

    let mut text_cmd_again = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    text_cmd_again.args(["doctor", "--format", "text"]);
    text_cmd_again.env("NO_COLOR", "1");
    let text_second = text_cmd_again.output().expect("doctor text no color");

    assert_eq!(text_first.status.code(), Some(0));
    assert_eq!(text_second.status.code(), Some(0));
    assert_eq!(text_first.stdout, text_second.stdout);
}

#[test]
fn inspect_and_doctor_agree_on_route_state_overlap_signals() {
    let inspect = parse_json(&run(&["inspect", "--format", "json", "--no-pretty"]).stdout);
    let doctor = parse_json(&run(&["doctor", "--format", "json", "--no-pretty"]).stdout);

    assert_eq!(inspect["status"], "ok");
    assert!(inspect["route_sources"].as_array().is_some_and(|rows| !rows.is_empty()));

    let checks = doctor["checks"].as_array().expect("doctor checks array");
    assert!(checks.iter().any(|v| v.as_str() == Some("routing")));
}

#[test]
fn dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution() {
    let temp = temp_dir("resolution");
    let cfg = temp.join("custom.env");
    fs::write(&cfg, "BIJUXCLI_ALPHA=1\n").expect("write config");

    let env_output = run_env(
        &["dev", "cli", "env", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_CONFIG", &cfg)],
    );
    assert_eq!(env_output.status.code(), Some(0));
    let env_json = parse_json(&env_output.stdout);
    assert_eq!(env_json["active"]["config_file"], cfg.to_string_lossy().to_string());
    assert_eq!(
        env_json["source_precedence"],
        serde_json::json!(["flags", "env", "config", "defaults"])
    );

    let contracts =
        parse_json(&run(&["dev", "cli", "contracts", "--format", "json", "--no-pretty"]).stdout);
    let contracts_snapshot: Value =
        serde_json::from_str(include_str!("snapshots/ported/dev_cli_contracts.json"))
            .expect("contracts snapshot json");
    assert_eq!(contracts["schema_version"], contracts_snapshot["schema_version"]);
    assert_eq!(contracts["contracts"], contracts_snapshot["contracts"]);

    let routes =
        parse_json(&run(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]).stdout);
    let routes_snapshot: Value =
        serde_json::from_str(include_str!("snapshots/ported/dev_cli_routes.json"))
            .expect("routes snapshot json");
    let snapshot_routes: BTreeSet<String> = routes_snapshot["routes"]
        .as_array()
        .expect("snapshot routes")
        .iter()
        .map(|row| {
            row["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    let current_routes: BTreeSet<String> = routes["routes"]
        .as_array()
        .expect("current routes")
        .iter()
        .map(|row| {
            row["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    assert!(snapshot_routes.is_subset(&current_routes));

    let registry =
        parse_json(&run(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]).stdout);
    let ownership = registry["ownership"].as_object().expect("ownership object");
    let plugin_names: BTreeSet<String> = ownership
        .get("plugin")
        .and_then(Value::as_array)
        .expect("plugin ownership list")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();
    let registry_names: BTreeSet<String> = registry["registry"]
        .as_array()
        .expect("registry rows")
        .iter()
        .filter(|row| row["owner"] == "plugin")
        .filter_map(|row| row["name"].as_str())
        .map(ToString::to_string)
        .collect();
    assert_eq!(plugin_names, registry_names);
}

#[test]
fn state_doctor_and_plugin_health_match_corruption_harness_findings() {
    let temp = temp_dir("plugin-corruption");
    let plugins_dir = temp.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::write(plugins_dir.join("registry.json"), "{\"version\":\"v1\",").expect("partial registry");

    let state_doctor = run_env(
        &["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", &plugins_dir)],
    );
    assert_eq!(state_doctor.status.code(), Some(0));
    let state_json = parse_json(&state_doctor.stdout);
    let issues = state_json["doctor"]["issues"].as_array().expect("issues");
    assert_eq!(state_json["doctor"]["status"], "degraded");
    assert!(!issues.is_empty());

    let mut plugin_health_cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    plugin_health_cmd.args(["dev", "cli", "plugin-health", "--format", "json", "--no-pretty"]);
    plugin_health_cmd.env("BIJUXCLI_PLUGINS_DIR", &plugins_dir);
    let plugin_health = plugin_health_cmd.output().expect("plugin health");
    assert_eq!(plugin_health.status.code(), Some(0));
    let plugin_health_json = parse_json(&plugin_health.stdout);
    let text_report =
        plugin_health_json["machine_report"]["text_report"].as_str().unwrap_or_default();
    assert!(text_report.contains("status: degraded"));
    assert_eq!(state_json["doctor"]["status"], "degraded");
}

#[test]
fn package_health_and_runtime_identity_are_consistent_with_active_binary_conditions() {
    let package_health = parse_json(
        &run(&["dev", "cli", "package-health", "--format", "json", "--no-pretty"]).stdout,
    );
    let runtime_identity = parse_json(
        &run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"]).stdout,
    );

    assert_eq!(runtime_identity["canonical_user_binary"], "bijux");
    let path_binaries = runtime_identity["path_binaries"].as_array().expect("path binaries");
    assert!(!path_binaries.is_empty());
    assert_eq!(runtime_identity["active_binary"], path_binaries[0]);

    let path_shadowing_detected = runtime_identity["diagnostics"]["path_shadowing_detected"]
        .as_bool()
        .expect("path shadowing bool");
    let summary = package_health["install_state_assumptions"]
        .as_array()
        .expect("assumptions")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    if path_shadowing_detected {
        assert!(summary.contains("PATH order decides active bijux binary"));
    }
}
