use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::Value;
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

fn workflow_graph(root: &Path) -> PathBuf {
    root.join("evidence/dag/authoring/examples/audience-branch-bulletin.dag.json")
}

fn copy_source_note(root: &Path, destination: &Path) -> PathBuf {
    let source = root.join("evidence/dag/authoring/examples/audience-branch-source/team-update.md");
    fs::create_dir_all(destination).expect("inputs dir");
    let note = destination.join("team-update.md");
    fs::copy(source, &note).expect("copy note");
    note
}

fn bulletin_path(run_dir: &Path) -> PathBuf {
    run_dir.join("nodes").join("publish_bulletin").join("outputs").join("publish").join("bulletin.md")
}

fn selection_path(run_dir: &Path) -> PathBuf {
    run_dir.join("nodes").join("publish_bulletin").join("outputs").join("publish").join("selection.json")
}

#[test]
fn audience_branch_workflow_selects_one_lane_and_records_join_behavior() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let note = copy_source_note(&root, &temp.path().join("inputs"));
    let runs_dir = temp.path().join("runs");
    fs::create_dir_all(&runs_dir).expect("runs dir");

    let graph = workflow_graph(&root);
    let payload = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "audience-branch-technical".to_string(),
            "--input".to_string(),
            format!("source_note={}", output_path_string(&note)),
            "--input".to_string(),
            "audience_mode=technical".to_string(),
        ],
        &root,
    );

    let run_dir = run_dir_from_response(&payload);
    let manifest = read_manifest(&run_dir);
    assert_eq!(manifest["status"], "success");
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["source_note"], output_path_string(&note));
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["audience_mode"], "technical");
    assert_eq!(manifest["node_counts"]["success"], 4);
    assert_eq!(manifest["node_counts"]["skipped"], 1);

    let choose = read_trace(&run_dir, "choose_audience_lane");
    let executive = read_trace(&run_dir, "render_executive_bulletin");
    let technical = read_trace(&run_dir, "render_technical_bulletin");
    let publish = read_trace(&run_dir, "publish_bulletin");

    assert_eq!(choose["status"], "success");
    assert_eq!(choose["branch_decision"], "technical");
    assert_eq!(executive["status"], "skipped");
    assert_eq!(executive["skip_reason"]["reason"], "branch_decision_not_selected");
    assert_eq!(technical["status"], "success");
    assert_eq!(publish["status"], "success");
    assert_eq!(publish["trigger_evaluation"]["trigger_rule"], "none_failed");
    assert_eq!(publish["trigger_evaluation"]["satisfied"], true);

    let parent_statuses = publish["trigger_evaluation"]["parent_statuses"].as_array().expect("parent statuses");
    assert!(parent_statuses.iter().any(|status| {
        status["node_id"] == "render_executive_bulletin" && status["status"] == "skipped"
    }));
    assert!(parent_statuses.iter().any(|status| {
        status["node_id"] == "render_technical_bulletin" && status["status"] == "success"
    }));

    let selection: Value =
        serde_json::from_str(&fs::read_to_string(selection_path(&run_dir)).expect("selection")).expect("selection json");
    assert_eq!(selection["selected_lane"], "technical");

    let bulletin = fs::read_to_string(bulletin_path(&run_dir)).expect("bulletin");
    assert!(bulletin.starts_with("# Technical Bulletin\n"));
    assert!(bulletin.contains("Audience lane: technical"));
    assert!(bulletin.contains("published the container-backed release note workflow"));
}
