#![forbid(unsafe_code)]
//! Contracts for `dev cli python *` control-plane reports.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_ok_json(args: &[&str]) -> Value {
    let out = run(args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("valid json")
}

#[test]
fn python_sovereignty_reports_have_stable_json_shapes() {
    let commands = [
        (["dev", "cli", "python", "bridge-status"], "bridge_status"),
        (["dev", "cli", "python", "surface-status"], "surface_status"),
        (["dev", "cli", "python", "sovereignty-audit"], "python_sovereignty_audit"),
        (["dev", "cli", "python", "drift"], "drift"),
        (["dev", "cli", "python", "packaging"], "packaging"),
    ];
    for (command, key) in commands {
        let first = run_ok_json(&command);
        let second = run_ok_json(&command);
        assert!(first.get(key).is_some(), "missing key {key} for {:?}", command);
        assert_eq!(
            first.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()),
            second.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()),
            "top-level keys changed for {:?}",
            command
        );
    }
}

#[test]
fn python_desovereignization_text_head_matches_snapshot() {
    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data").join("golden").join("cli_surface")
        .join("dev_cli_python_desovereignization_text_head.txt");
    let expected_head = fs::read_to_string(snapshot_path).expect("read snapshot");
    let out = run(&["dev", "cli", "python", "sovereignty-audit", "--format", "text"]);
    assert!(out.status.success(), "python sovereignty text command failed");
    let text = String::from_utf8(out.stdout).expect("utf8");
    assert!(text.starts_with(&expected_head), "python sovereignty text snapshot drift");
}

#[test]
fn python_text_heads_match_snapshots() {
    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data").join("golden").join("cli_surface")
        .join("dev_cli_python_text_heads.json");
    let expected: BTreeMap<String, String> =
        serde_json::from_str(&fs::read_to_string(snapshot_path).expect("read snapshot"))
            .expect("parse snapshot");

    for (command, prefix) in expected {
        let mut args: Vec<&str> = command.split_whitespace().collect();
        args.push("--format");
        args.push("text");
        let out = run(&args);
        assert!(out.status.success(), "text command failed for {command}");
        let text = String::from_utf8(out.stdout).expect("utf8");
        assert!(text.starts_with(&prefix), "snapshot drift for {command}");
    }
}
