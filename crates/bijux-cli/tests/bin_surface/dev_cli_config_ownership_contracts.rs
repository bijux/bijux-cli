#![forbid(unsafe_code)]
//! Contracts for `dev cli config *` ownership surfaces.

use std::process::Command;
use std::{collections::BTreeMap, fs, path::Path};

use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
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
fn config_ownership_json_contracts_are_stable() {
    let commands = [
        (["dev", "cli", "config", "rust-owner"], "rust_owner"),
        (["dev", "cli", "config", "python-owner"], "python_owner"),
        (["dev", "cli", "config", "ownership"], "owners"),
        (["dev", "cli", "config", "drift"], "drift"),
        (["dev", "cli", "config", "shape"], "schemas"),
        (["dev", "cli", "config", "evidence-map"], "evidence_ids"),
    ];

    for (command, key) in commands {
        let first = run_ok_json(&command);
        let second = run_ok_json(&command);
        assert!(
            first.get(key).is_some(),
            "missing key {key} for {:?}",
            command
        );
        assert_eq!(
            first
                .as_object()
                .map(|o| o.keys().cloned().collect::<Vec<_>>()),
            second
                .as_object()
                .map(|o| o.keys().cloned().collect::<Vec<_>>()),
            "top-level keys changed for {:?}",
            command
        );
    }
}

#[test]
fn config_ownership_text_outputs_are_non_empty_and_structured() {
    let commands = [
        "dev cli config rust-owner",
        "dev cli config python-owner",
        "dev cli config ownership",
        "dev cli config drift",
        "dev cli config shape",
        "dev cli config evidence-map",
    ];
    for command in commands {
        let mut args: Vec<&str> = command.split_whitespace().collect();
        args.push("--format");
        args.push("text");
        let out = run(&args);
        assert!(out.status.success(), "text command failed for {command}");
        let text = String::from_utf8(out.stdout).expect("utf8");
        assert!(!text.trim().is_empty(), "text output empty for {command}");
        assert!(
            text.contains('{') || text.contains('['),
            "text output should remain structured for {command}"
        );
    }
}

#[test]
fn config_ownership_text_heads_match_snapshot() {
    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join("dev_cli_config_ownership_text_heads.json");
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
