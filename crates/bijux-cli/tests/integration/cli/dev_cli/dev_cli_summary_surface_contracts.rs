#![forbid(unsafe_code)]
//! Contracts for top-level maintainer summary commands.

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

fn run_ok_json(command: &[&str]) -> Value {
    let mut args = command.to_vec();
    args.push("--format");
    args.push("json");
    args.push("--no-pretty");
    let out = run(&args, &[]);
    assert!(
        out.status.success(),
        "json command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("json payload")
}

fn assert_text_non_empty(command: &[&str]) {
    let mut args = command.to_vec();
    args.push("--format");
    args.push("text");
    let out = run(&args, &[]);
    assert!(
        out.status.success(),
        "text command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).expect("utf8");
    assert!(!text.trim().is_empty(), "text output must be non-empty for {:?}", command);
}

#[test]
fn status_dashboard_truth_blockers_next_json_contracts() {
    for command in [
        ["dev", "cli", "status"],
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ] {
        let payload = run_ok_json(&command);
        assert!(payload.is_object(), "payload must be an object for {:?}", command);
    }
}

#[test]
fn status_dashboard_truth_blockers_next_text_contracts() {
    for command in [
        ["dev", "cli", "status"],
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ] {
        assert_text_non_empty(&command);
    }
}

#[test]
fn status_and_truth_counts_are_consistent() {
    let status = run_ok_json(&["dev", "cli", "status"]);
    let truth = run_ok_json(&["dev", "cli", "truth"]);

    let status_summary = &status["status_report"]["summary"];
    let truth_done = truth["truth"]["done"]["summary"]["count"].as_u64().expect("truth done count");
    let truth_missing =
        truth["truth"]["missing"]["summary"]["count"].as_u64().expect("truth missing count");
    let truth_partial =
        truth["truth"]["partial"]["summary"]["count"].as_u64().expect("truth partial count");
    let truth_intentional = truth["truth"]["intentional_differences"]["summary"]["count"]
        .as_u64()
        .expect("truth intentional count");

    assert_eq!(status_summary["complete"].as_u64(), Some(truth_done));
    assert_eq!(status_summary["missing"].as_u64(), Some(truth_missing));
    assert_eq!(
        status_summary["partial"].as_u64().unwrap_or_default()
            + status_summary["shim"].as_u64().unwrap_or_default(),
        truth_partial + truth_intentional
    );
}

#[test]
fn blockers_is_subset_of_unresolved_status_data() {
    let status = run_ok_json(&["dev", "cli", "status"]);
    let blockers = run_ok_json(&["dev", "cli", "blockers"]);

    let unresolved: std::collections::BTreeSet<String> = status["status_report"]["commands"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|row| row.get("status") != Some(&Value::String("complete".to_string())))
        .filter_map(|row| row.get("command").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect();

    let blocker_rows = blockers["blockers"].as_array().cloned().unwrap_or_default();
    for row in blocker_rows {
        let command = row
            .get("command")
            .and_then(Value::as_str)
            .or_else(|| row.as_str())
            .unwrap_or_default()
            .to_string();
        if command.is_empty() {
            continue;
        }
        assert!(
            unresolved.contains(&command),
            "blocker command `{command}` must exist in unresolved status data"
        );
    }
}

#[test]
fn next_is_generated_from_evidence_and_status_inputs() {
    let next = run_ok_json(&["dev", "cli", "next"]);
    let policy = &next["next"]["minimalism"]["evidence_first_policy"];
    assert_eq!(policy["manual_curated_priority_lists_allowed"], Value::Bool(false));
    assert_eq!(policy["roadmap_requires_generated_artifacts"], Value::Bool(true));
    assert!(
        policy["required_artifacts"].as_array().is_some_and(|rows| !rows.is_empty()),
        "next command must declare generated artifact inputs"
    );
}

#[test]
fn dashboard_reflects_same_status_summary_as_standalone_status() {
    let status = run_ok_json(&["dev", "cli", "status"]);
    let dashboard = run_ok_json(&["dev", "cli", "dashboard"]);
    assert_eq!(dashboard["dashboard"]["status"]["summary"], status["status_report"]["summary"]);
}

#[test]
fn summary_commands_work_with_missing_optional_state_paths() {
    let root = std::env::temp_dir().join(format!("bijux-summary-missing-{}", std::process::id()));
    fs::create_dir_all(&root).expect("mkdir");
    let envs = [
        ("BIJUX_CONFIG_PATH", root.join("missing-config.env").to_string_lossy().to_string()),
        ("BIJUX_HISTORY_PATH", root.join("missing-history.json").to_string_lossy().to_string()),
        ("BIJUX_MEMORY_PATH", root.join("missing-memory.json").to_string_lossy().to_string()),
    ];
    for command in [
        ["dev", "cli", "status"],
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ] {
        let out = run(&command, &envs);
        assert!(out.status.success(), "command failed with missing optional state: {:?}", command);
    }
}

#[test]
fn summary_commands_work_with_corrupted_optional_state() {
    let root = std::env::temp_dir().join(format!("bijux-summary-corrupt-{}", std::process::id()));
    fs::create_dir_all(&root).expect("mkdir");
    let config = root.join("config.env");
    let history = root.join("history.json");
    let memory = root.join("memory.json");
    fs::write(&config, "BROKEN=\0\n").expect("write config");
    fs::write(&history, "{not-json").expect("write history");
    fs::write(&memory, "{not-json").expect("write memory");
    let envs = [
        ("BIJUX_CONFIG_PATH", config.to_string_lossy().to_string()),
        ("BIJUX_HISTORY_PATH", history.to_string_lossy().to_string()),
        ("BIJUX_MEMORY_PATH", memory.to_string_lossy().to_string()),
    ];
    for command in [
        ["dev", "cli", "status"],
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ] {
        let out = run(&command, &envs);
        assert!(
            out.status.success(),
            "command failed with corrupted optional state: {:?}",
            command
        );
    }
}

#[test]
fn summary_commands_tolerate_old_artifact_timestamps() {
    let stale = std::env::temp_dir().join(format!("bijux-stale-marker-{}", std::process::id()));
    fs::write(&stale, "stale-marker").expect("write stale marker");
    let envs = [("BIJUX_STALE_MARKER_PATH", stale.to_string_lossy().to_string())];
    for command in [
        ["dev", "cli", "status"],
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ] {
        let out = run(&command, &envs);
        assert!(out.status.success(), "command failed under stale artifact marker: {:?}", command);
    }
}
