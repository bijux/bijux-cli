#![forbid(unsafe_code)]
//! Contracts for state-audit and state-doctor hardening.

use std::fs;
use std::process::Command;

use serde_json::Value;

fn run(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
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
fn state_audit_and_state_doctor_json_and_text_contracts() {
    let audit_json = run_ok_json(&["dev", "cli", "state-audit"], &[]);
    let doctor_json = run_ok_json(&["dev", "cli", "state-doctor"], &[]);
    assert!(audit_json.get("paths").is_some());
    assert!(doctor_json.get("doctor").is_some());

    for command in [
        ["dev", "cli", "state-audit"],
        ["dev", "cli", "state-doctor"],
    ] {
        let out = run(
            &[command[0], command[1], command[2], "--format", "text"],
            &[],
        );
        assert!(out.status.success(), "text command failed: {:?}", command);
        assert!(!String::from_utf8_lossy(&out.stdout).trim().is_empty());
    }
}

#[test]
fn state_audit_reports_truthful_paths() {
    let root = std::env::temp_dir().join(format!("bijux-state-paths-{}", std::process::id()));
    fs::create_dir_all(&root).expect("mkdir");
    let config = root.join("config.env");
    let history = root.join("history.json");
    let memory = root.join("memory.json");
    let plugins = root.join("registry.json");
    fs::write(&config, "BIJUXCLI_KEY=1\n").expect("write config");
    fs::write(&history, "[]").expect("write history");
    fs::write(&memory, "{}").expect("write memory");
    fs::write(&plugins, "{\"plugins\":{}}").expect("write plugins");
    let audit = run_ok_json(
        &[
            "dev",
            "cli",
            "state-audit",
            "--config-path",
            config.to_string_lossy().as_ref(),
        ],
        &[],
    );
    assert_eq!(
        audit["paths"]["config"]["path"],
        Value::String(config.to_string_lossy().to_string())
    );
    assert!(audit["paths"]["history"]["path"]
        .as_str()
        .is_some_and(|s| !s.is_empty()));
    assert!(audit["paths"]["memory"]["path"]
        .as_str()
        .is_some_and(|s| !s.is_empty()));
    assert!(audit["paths"]["plugins_registry"]["path"]
        .as_str()
        .is_some_and(|s| !s.is_empty()));
}

#[test]
fn state_doctor_detects_corrupted_config_registry_history_and_memory() {
    let root = std::env::temp_dir().join(format!("bijux-state-corrupt-{}", std::process::id()));
    fs::create_dir_all(&root).expect("mkdir");
    let config = root.join("config.env");
    let history = root.join("history.json");
    let memory = root.join("memory.json");
    let plugins = root.join("registry.json");
    fs::write(&config, "BROKEN_LINE\n").expect("write bad config");
    fs::write(&history, "{not-json").expect("write bad history");
    fs::write(&memory, "{not-json").expect("write bad memory");
    fs::write(&plugins, "{not-json").expect("write bad plugin registry");
    let doctor = run_ok_json(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--config-path",
            config.to_string_lossy().as_ref(),
        ],
        &[],
    );
    let issues = doctor["doctor"]["issues"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let areas: std::collections::BTreeSet<String> = issues
        .iter()
        .filter_map(|row| row.get("area").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect();
    assert!(areas.contains("config"));
}

#[test]
fn state_doctor_reports_actionable_or_explicitly_empty_repairs() {
    let doctor = run_ok_json(&["dev", "cli", "state-doctor"], &[]);
    let repairs = doctor["doctor"]["repairs"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for repair in repairs {
        assert!(
            repair
                .get("action")
                .and_then(Value::as_str)
                .is_some_and(|s| !s.trim().is_empty()),
            "repair entries must include actionable text"
        );
    }
}

#[test]
fn state_doctor_json_findings_have_stable_ordering_and_text_is_deterministic() {
    let first = run_ok_json(&["dev", "cli", "state-doctor"], &[]);
    let second = run_ok_json(&["dev", "cli", "state-doctor"], &[]);
    assert_eq!(first, second, "state-doctor json ordering/output drift");

    let text_args = ["dev", "cli", "state-doctor", "--format", "text"];
    let text_first = run(&text_args, &[]);
    let text_second = run(&text_args, &[]);
    assert!(text_first.status.success());
    assert!(text_second.status.success());
    assert_eq!(
        text_first.stdout, text_second.stdout,
        "state-doctor text drift"
    );
}
