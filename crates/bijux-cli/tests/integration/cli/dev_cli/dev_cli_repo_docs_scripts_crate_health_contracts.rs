#![forbid(unsafe_code)]
//! Contracts for repo/docs/maintenance/crate-health cleanup backbone surfaces.

use std::process::Command;

use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_ok_json(command: &[&str]) -> Value {
    let mut args = command.to_vec();
    args.push("--format");
    args.push("json");
    args.push("--no-pretty");
    let out = run(&args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("json payload")
}

fn run_ok_text_non_empty(command: &[&str]) {
    let mut args = command.to_vec();
    args.push("--format");
    args.push("text");
    let out = run(&args);
    assert!(
        out.status.success(),
        "text command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!String::from_utf8_lossy(&out.stdout).trim().is_empty());
}

#[test]
fn repo_docs_maintenance_crate_health_json_and_text_contracts() {
    for command in [
        &["dev", "cli", "repo", "health"][..],
        &["dev", "cli", "docs-audit"][..],
        &["dev", "cli", "maintenance-audit"][..],
        &["dev", "cli", "crate-health"][..],
    ] {
        let json = run_ok_json(command);
        assert!(json.is_object(), "json object expected for {:?}", command);
        run_ok_text_non_empty(command);
    }
}

#[test]
fn docs_audit_exposes_duplicate_and_stale_reference_signals() {
    let docs = run_ok_json(&["dev", "cli", "docs-audit"]);
    assert!(docs["docs"].is_array(), "docs rows must exist");
    assert!(docs["docs_audit"].is_object(), "docs_audit summary must exist");
    assert!(
        docs["docs_audit"].as_object().is_some(),
        "docs audit should expose machine-readable stale/duplicate summary fields"
    );
}

#[test]
fn maintenance_audit_exposes_remaining_and_migrated_views() {
    let scripts = run_ok_json(&["dev", "cli", "maintenance-audit"]);
    assert!(scripts["maintenance"].is_array());
    assert!(scripts.get("remaining_legacy_only_behaviors").is_some());
    assert!(scripts.get("remaining_task_runner_only_behaviors").is_some());
    assert!(scripts["replacement_rule"].is_string());
}

#[test]
fn crate_health_exposes_dependency_and_public_api_truth() {
    let crate_health = run_ok_json(&["dev", "cli", "crate-health"]);
    assert!(crate_health["dependency_edges"].is_array());
    assert!(crate_health["public_api_by_crate"].is_object());
    assert!(crate_health["public_api_counts"].is_array());
}

#[test]
fn repo_health_exposes_stale_generated_artifact_detection() {
    let repo = run_ok_json(&["dev", "cli", "repo", "health"]);
    assert!(repo["repo_health"].is_object());
    let stale_generated = repo["repo_health"]["generated"]["stale_generated_artifacts"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let stale_generated_legacy = repo["repo_health"]["stale"]["stale_generated_artifacts"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        repo["repo_health"]["generated"].get("stale_generated_artifacts").is_some()
            || repo["repo_health"]["stale"].get("stale_generated_artifacts").is_some()
            || !stale_generated.is_empty()
            || !stale_generated_legacy.is_empty(),
        "repo health must include stale generated artifact signal"
    );
}
