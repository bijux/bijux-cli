use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = support::run_dag_command(args, cwd);
    assert_eq!(code, 0, "command failed: stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn run_json_owned(args: Vec<String>, cwd: &Path) -> Value {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_json(&refs, cwd)
}

fn run_json_owned_allow_failure(args: Vec<String>, cwd: &Path) -> (i32, Value, String) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (code, stdout, stderr) = support::run_dag_command(&refs, cwd);
    let payload = serde_json::from_str(&stdout).expect("parse json envelope");
    (code, payload, stderr)
}

fn run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run dir"))
}

fn read_manifest(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
        .expect("manifest json")
}

fn read_trace(run_dir: &Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json")).expect("trace"),
    )
    .expect("trace json")
}

fn read_attempts(run_dir: &Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("attempts.json"))
            .expect("attempts"),
    )
    .expect("attempts json")
}

fn workflow_graph(root: &Path) -> PathBuf {
    root.join("evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json")
}

fn copy_source_note(root: &Path, destination: &Path) -> PathBuf {
    let source =
        root.join("evidence/dag/authoring/examples/compliance-gated-source/team-update.md");
    fs::create_dir_all(destination).expect("inputs dir");
    let note = destination.join("team-update.md");
    fs::copy(source, &note).expect("copy note");
    note
}

fn write_retry_plan(path: &Path, fail_until_attempt: u64) {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "fail_until_attempt": fail_until_attempt,
            "gate_policy": "manual-approval",
            "expected_reviewer_group": "release-managers"
        }))
        .expect("retry plan"),
    )
    .expect("write retry plan");
}

fn write_publication_gate(path: &Path, approved: bool, reviewer: &str, reviewer_group: &str) {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "approved": approved,
            "reviewer": reviewer,
            "reviewer_group": reviewer_group
        }))
        .expect("publication gate"),
    )
    .expect("write publication gate");
}

fn bulletin_path(run_dir: &Path) -> PathBuf {
    run_dir
        .join("nodes")
        .join("publish_bulletin")
        .join("outputs")
        .join("publish")
        .join("bulletin.md")
}

fn node_stderr_path(run_dir: &Path, node_id: &str) -> PathBuf {
    run_dir.join("nodes").join(node_id).join("stderr.log")
}

#[test]
fn compliance_gated_bulletin_workflow_repairs_the_failed_publication_boundary() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let note = copy_source_note(&root, &temp.path().join("inputs"));
    let retry_plan = temp.path().join("retry-plan.json");
    let publication_gate = temp.path().join("publication-gate.json");
    let runs_dir = temp.path().join("runs");
    fs::create_dir_all(&runs_dir).expect("runs dir");

    write_retry_plan(&retry_plan, 1);
    write_publication_gate(&publication_gate, false, "", "release-managers");

    let (code, source, stderr) = run_json_owned_allow_failure(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&workflow_graph(&root)),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "compliance-gated-source".to_string(),
            "--input".to_string(),
            format!("source_note={}", output_path_string(&note)),
            "--input".to_string(),
            format!("retry_plan={}", output_path_string(&retry_plan)),
            "--input".to_string(),
            format!("publication_gate={}", output_path_string(&publication_gate)),
            "--input".to_string(),
            "bulletin_title=Compliance Review Bulletin".to_string(),
        ],
        &root,
    );
    assert!(code == 0 || code == 3, "unexpected command code: {code} stderr={stderr}");

    let source_run = run_dir_from_response(&source);
    let source_manifest = read_manifest(&source_run);
    assert_eq!(source_manifest["status"], "failed");
    assert_eq!(source_manifest["node_counts"]["success"], 2);
    assert_eq!(source_manifest["node_counts"]["failed"], 2);
    assert_eq!(source_manifest["node_counts"]["skipped"], 0);

    let summary = &source["data"]["summary"];
    assert_eq!(summary["status"], "failed");
    let failed_node_reasons = summary["failed_node_reasons"]
        .as_array()
        .expect("failed node reasons array");
    assert!(failed_node_reasons.iter().any(|reason| {
        reason["node_id"] == "validate_publication_gate" && reason["code"] == "EXEC_FAIL"
    }));
    assert!(failed_node_reasons.iter().any(|reason| {
        reason["node_id"] == "publish_bulletin" && reason["code"] == "UPSTREAM_FAILED"
    }));

    let fetch_trace = read_trace(&source_run, "fetch_compliance_gate");
    assert_eq!(fetch_trace["status"], "success");
    assert_eq!(fetch_trace["attempt"], 2);

    let fetch_attempts = read_attempts(&source_run, "fetch_compliance_gate");
    let fetch_attempts = fetch_attempts.as_array().expect("attempt array");
    assert_eq!(fetch_attempts.len(), 2);
    assert_eq!(fetch_attempts[0]["status"], "Failed");
    assert_eq!(fetch_attempts[0]["scheduled_backoff_ms"], 10);
    assert_eq!(fetch_attempts[1]["status"], "Success");

    let validate_trace = read_trace(&source_run, "validate_publication_gate");
    let publish_trace = read_trace(&source_run, "publish_bulletin");
    assert_eq!(validate_trace["status"], "failed");
    assert_eq!(publish_trace["status"], "failed");
    assert_eq!(publish_trace["failure"]["code"], "UPSTREAM_FAILED");
    assert!(fs::read_to_string(node_stderr_path(&source_run, "validate_publication_gate"))
        .expect("validate stderr")
        .contains("publication gate is not approved"));

    let explain_failure = run_json_owned(
        vec![
            "--json".to_string(),
            "runs".to_string(),
            "explain-failure".to_string(),
            "compliance-gated-source".to_string(),
            "--root".to_string(),
            output_path_string(&runs_dir),
        ],
        &root,
    );
    assert_eq!(explain_failure["data"]["root_failure"], "validate_publication_gate");
    let propagated_failures = explain_failure["data"]["propagated_failures"]
        .as_array()
        .expect("propagated failures");
    assert!(propagated_failures.iter().any(|entry| entry["node_id"] == "publish_bulletin"));
    assert_eq!(
        explain_failure["data"]["propagated_skips"].as_array().expect("propagated skips").len(),
        0
    );

    write_publication_gate(&publication_gate, true, "A. Reviewer", "release-managers");

    let replay = run_json_owned(
        vec![
            "replay".to_string(),
            "--json".to_string(),
            "--source-run-id".to_string(),
            "compliance-gated-source".to_string(),
            "--source-run-root".to_string(),
            output_path_string(&runs_dir),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "compliance-gated-repaired".to_string(),
            "--from-node".to_string(),
            "validate_publication_gate".to_string(),
        ],
        &root,
    );

    let replay_run = run_dir_from_response(&replay);
    let replay_manifest = read_manifest(&replay_run);
    assert_eq!(replay_manifest["status"], "success");
    assert_eq!(replay_manifest["run_metadata"]["parent_run_id"], "compliance-gated-source");
    assert_eq!(replay_manifest["run_metadata"]["source_run_id"], "compliance-gated-source");
    assert_eq!(replay["data"]["node_rerun_diff"]["node_id"], "validate_publication_gate");

    let replay_validate = read_trace(&replay_run, "validate_publication_gate");
    let replay_publish = read_trace(&replay_run, "publish_bulletin");
    assert_eq!(replay_validate["status"], "success");
    assert_eq!(replay_publish["status"], "success");

    let bulletin = fs::read_to_string(bulletin_path(&replay_run)).expect("bulletin");
    assert!(bulletin.contains("# Compliance Review Bulletin"));
    assert!(bulletin.contains("Approved by: A. Reviewer"));
    assert!(bulletin.contains("Gate lookup attempt: 2"));

    let verify =
        run_json(&["verify", "--json", &output_path_string(&replay_run), "--strict"], &root);
    assert_eq!(verify["ok"], true);
    assert_eq!(verify["data"]["event_log_completeness"]["complete"], true);
}

#[test]
fn compliance_gated_bulletin_workflow_surfaces_retry_exhaustion() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let note = copy_source_note(&root, &temp.path().join("inputs"));
    let retry_plan = temp.path().join("retry-plan.json");
    let publication_gate = temp.path().join("publication-gate.json");
    let runs_dir = temp.path().join("runs");
    fs::create_dir_all(&runs_dir).expect("runs dir");

    write_retry_plan(&retry_plan, 9);
    write_publication_gate(&publication_gate, true, "A. Reviewer", "release-managers");

    let (code, payload, stderr) = run_json_owned_allow_failure(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&workflow_graph(&root)),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "compliance-gated-exhausted".to_string(),
            "--input".to_string(),
            format!("source_note={}", output_path_string(&note)),
            "--input".to_string(),
            format!("retry_plan={}", output_path_string(&retry_plan)),
            "--input".to_string(),
            format!("publication_gate={}", output_path_string(&publication_gate)),
        ],
        &root,
    );
    assert!(code == 0 || code == 3, "unexpected command code: {code} stderr={stderr}");

    let run_dir = run_dir_from_response(&payload);
    let manifest = read_manifest(&run_dir);
    assert_eq!(manifest["status"], "failed");
    assert_eq!(manifest["node_counts"]["success"], 1);
    assert_eq!(manifest["node_counts"]["failed"], 3);
    assert_eq!(manifest["node_counts"]["skipped"], 0);

    let summary = &payload["data"]["summary"];
    let failed_node_reasons = summary["failed_node_reasons"]
        .as_array()
        .expect("failed node reasons array");
    assert!(failed_node_reasons.iter().any(|reason| {
        reason["node_id"] == "fetch_compliance_gate" && reason["code"] == "EXEC_FAIL"
    }));

    let fetch_trace = read_trace(&run_dir, "fetch_compliance_gate");
    assert_eq!(fetch_trace["status"], "failed");
    assert_eq!(fetch_trace["attempt"], 3);
    assert!(fs::read_to_string(node_stderr_path(&run_dir, "fetch_compliance_gate"))
        .expect("fetch stderr")
        .contains("transient compliance gate lookup failed on attempt 3"));

    let fetch_attempts = read_attempts(&run_dir, "fetch_compliance_gate");
    let fetch_attempts = fetch_attempts.as_array().expect("attempt array");
    assert_eq!(fetch_attempts.len(), 3);
    assert_eq!(fetch_attempts[0]["status"], "Failed");
    assert_eq!(fetch_attempts[1]["status"], "Failed");
    assert_eq!(fetch_attempts[2]["status"], "Failed");
    assert_eq!(fetch_attempts[2]["retry_decision"]["reason"], "retry_budget_exhausted");

    let explain_failure = run_json_owned(
        vec![
            "--json".to_string(),
            "runs".to_string(),
            "explain-failure".to_string(),
            "compliance-gated-exhausted".to_string(),
            "--root".to_string(),
            output_path_string(&runs_dir),
        ],
        &root,
    );
    assert_eq!(explain_failure["data"]["root_failure"], "fetch_compliance_gate");
    let propagated_failures = explain_failure["data"]["propagated_failures"]
        .as_array()
        .expect("propagated failures");
    assert!(propagated_failures.iter().any(|entry| entry["node_id"] == "validate_publication_gate"));
    assert!(propagated_failures.iter().any(|entry| entry["node_id"] == "publish_bulletin"));
    assert_eq!(
        explain_failure["data"]["propagated_skips"].as_array().expect("propagated skips").len(),
        0
    );
}
