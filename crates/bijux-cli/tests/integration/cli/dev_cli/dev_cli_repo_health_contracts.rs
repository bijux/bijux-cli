#![forbid(unsafe_code)]
//! Contracts for `dev cli repo *` maintainer health surfaces.

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
fn repo_health_json_contracts_are_stable() {
    let commands = [
        (["dev", "cli", "repo", "health"], "repo_health"),
        (["dev", "cli", "repo", "drift"], "dead_scripts_references"),
        (["dev", "cli", "repo", "inventories"], "stale_inventories"),
        (["dev", "cli", "repo", "generated"], "stale_generated_artifacts"),
        (["dev", "cli", "repo", "stale"], "stale_snapshots"),
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
fn repo_text_heads_match_snapshots() {
    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("golden")
        .join("cli_surface")
        .join("dev_cli_repo_text_heads.json");
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
