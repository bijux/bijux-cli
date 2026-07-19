use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_internal_json_owned(args: Vec<String>, cwd: &Path) -> Value {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (code, stdout, stderr) =
        support::run_dag_command_with_env(&refs, cwd, &[("BIJUX_DAG_ENABLE_INTERNAL", "1")]);
    assert_eq!(code, 0, "command failed: stdout={stdout} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
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

fn repair_run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["repair_run"]["run_dir"].as_str().expect("repair run dir"))
}

fn read_manifest(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
        .expect("manifest json")
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
        serde_json::to_vec_pretty(&serde_json::json!({
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
        serde_json::to_vec_pretty(&serde_json::json!({
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

fn build_failed_publication_run(
    root: &Path,
    temp_root: &Path,
) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let note = copy_source_note(root, &temp_root.join("inputs"));
    let retry_plan = temp_root.join("retry-plan.json");
    let publication_gate = temp_root.join("publication-gate.json");
    let runs_dir = temp_root.join("runs");
    fs::create_dir_all(&runs_dir).expect("runs dir");

    write_retry_plan(&retry_plan, 1);
    write_publication_gate(&publication_gate, false, "", "release-managers");

    let (code, source, stderr) = run_json_owned_allow_failure(
        vec![
            "--json".to_string(),
            "run".to_string(),
            output_path_string(&workflow_graph(root)),
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
        root,
    );
    assert!(code == 0 || code == 3, "unexpected command code: {code} stderr={stderr}");
    (run_dir_from_response(&source), note, retry_plan, publication_gate, runs_dir)
}

fn repair_command_payload(run_dir: &Path, runs_dir: &Path, apply: bool) -> Value {
    let mut args = vec!["runtime".to_string(), "repair".to_string(), "--json".to_string()];
    if apply {
        args.push("--apply".to_string());
    }
    args.push("--out".to_string());
    args.push(output_path_string(runs_dir));
    if apply {
        args.push("--run-id".to_string());
        args.push("compliance-gated-repaired".to_string());
    }
    args.push(output_path_string(run_dir));
    run_internal_json_owned(args, &repo_root())
}

#[test]
fn runtime_repair_spawns_verified_child_run_for_failed_boundary() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let (source_run, _note, _retry_plan, publication_gate, runs_dir) =
        build_failed_publication_run(&root, temp.path());
    let source_manifest = read_manifest(&source_run);
    assert_eq!(source_manifest["status"], "failed");

    write_publication_gate(&publication_gate, true, "A. Reviewer", "release-managers");

    let repair = repair_command_payload(&source_run, &runs_dir, true);

    assert_eq!(repair["ok"], true);
    assert_eq!(
        repair["data"]["repair_roots"].as_array().expect("repair roots"),
        &vec![Value::String("validate_publication_gate".to_string())]
    );
    assert_eq!(repair["data"]["repair_run"]["verified"], true);
    assert_eq!(repair["data"]["repair_run"]["boundary_verification"]["verified"], true);

    let repair_run = repair_run_dir_from_response(&repair);
    let repair_manifest = read_manifest(&repair_run);
    assert_eq!(repair_manifest["status"], "success");
    assert_eq!(repair_manifest["run_metadata"]["parent_run_id"], "compliance-gated-source");
    assert_eq!(repair_manifest["run_metadata"]["source_run_id"], "compliance-gated-source");
    assert_eq!(repair["data"]["repair_run"]["verify_report"]["status"], "ok");
    assert_eq!(
        repair["data"]["repair_run"]["node_rerun_diffs"][0]["node_id"],
        "validate_publication_gate"
    );

    let bulletin = fs::read_to_string(bulletin_path(&repair_run)).expect("bulletin");
    assert!(bulletin.contains("# Compliance Review Bulletin"));
    assert!(bulletin.contains("Approved by: A. Reviewer"));
}
