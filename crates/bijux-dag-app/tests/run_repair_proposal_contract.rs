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

fn run_json_owned_allow_failure(args: Vec<String>, cwd: &Path) -> (i32, Value, String) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (code, stdout, stderr) = support::run_dag_command(&refs, cwd);
    let payload = serde_json::from_str(&stdout).expect("parse json envelope");
    (code, payload, stderr)
}

fn run_internal_json_owned_allow_failure(args: Vec<String>, cwd: &Path) -> (i32, Value, String) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (code, stdout, stderr) =
        support::run_dag_command_with_env(&refs, cwd, &[("BIJUX_DAG_ENABLE_INTERNAL", "1")]);
    let payload = serde_json::from_str(&stdout).expect("parse json envelope");
    (code, payload, stderr)
}

fn run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run dir"))
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

fn build_failed_publication_run(root: &Path, temp_root: &Path) -> (PathBuf, PathBuf) {
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
    (run_dir_from_response(&source), runs_dir)
}

fn build_successful_publication_run(root: &Path, temp_root: &Path) -> (PathBuf, PathBuf) {
    let note = copy_source_note(root, &temp_root.join("inputs"));
    let retry_plan = temp_root.join("retry-plan.json");
    let publication_gate = temp_root.join("publication-gate.json");
    let runs_dir = temp_root.join("runs");
    fs::create_dir_all(&runs_dir).expect("runs dir");

    write_retry_plan(&retry_plan, 0);
    write_publication_gate(&publication_gate, true, "A. Reviewer", "release-managers");

    let (code, source, stderr) = run_json_owned_allow_failure(
        vec![
            "--json".to_string(),
            "run".to_string(),
            output_path_string(&workflow_graph(root)),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "compliance-gated-success".to_string(),
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
    assert_eq!(code, 0, "successful run failed: stderr={stderr}");
    (run_dir_from_response(&source), runs_dir)
}

fn repair_command_payload_allow_failure(run_dir: &Path, runs_dir: &Path) -> (i32, Value, String) {
    let args = vec![
        "runtime".to_string(),
        "repair".to_string(),
        "--json".to_string(),
        "--out".to_string(),
        output_path_string(runs_dir),
        output_path_string(run_dir),
    ];
    run_internal_json_owned_allow_failure(args, &repo_root())
}

#[test]
fn runtime_repair_proposes_failed_boundary_actions_before_apply() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let (source_run, runs_dir) = build_failed_publication_run(&root, temp.path());

    let (code, repair, stderr) = repair_command_payload_allow_failure(&source_run, &runs_dir);
    assert_eq!(code, 3, "proposal command should fail with issues: stderr={stderr}");

    assert_eq!(repair["ok"], false);
    assert_eq!(
        repair["data"]["repair_roots"].as_array().expect("repair roots"),
        &vec![Value::String("validate_publication_gate".to_string())]
    );
    assert_eq!(
        repair["data"]["invalidated_nodes"].as_array().expect("invalidated nodes"),
        &vec![
            Value::String("publish_bulletin".to_string()),
            Value::String("validate_publication_gate".to_string())
        ]
    );
    assert!(repair["data"]["issues"].as_array().expect("repair issues").iter().any(|issue| {
        issue["kind"] == "failed_node" && issue["node_id"] == "validate_publication_gate"
    }));
    assert!(repair["data"]["proposed_actions"].as_array().expect("proposed actions").iter().any(
        |action| {
            action["kind"] == "rerun_downstream_closure"
                && action["node_roots"]
                    == Value::Array(vec![Value::String("validate_publication_gate".to_string())])
                && action["affected_nodes"]
                    == Value::Array(vec![
                        Value::String("publish_bulletin".to_string()),
                        Value::String("validate_publication_gate".to_string()),
                    ])
        }
    ));
    assert_eq!(repair["data"]["repair_run"], Value::Null);
}

#[test]
fn runtime_repair_detects_corrupt_artifact_in_successful_run() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let (run_dir, runs_dir) = build_successful_publication_run(&root, temp.path());

    fs::write(bulletin_path(&run_dir), "# Corrupted Bulletin\n").expect("corrupt bulletin");

    let (code, repair, stderr) = repair_command_payload_allow_failure(&run_dir, &runs_dir);
    assert_eq!(code, 3, "corruption proposal should fail with issues: stderr={stderr}");

    assert_eq!(repair["ok"], false);
    assert_eq!(
        repair["data"]["repair_roots"].as_array().expect("repair roots"),
        &vec![Value::String("publish_bulletin".to_string())]
    );
    assert!(repair["data"]["issues"].as_array().expect("issues").iter().any(|issue| {
        issue["kind"] == "corrupt_artifact"
            && issue["node_id"] == "publish_bulletin"
            && issue["output_name"] == "bulletin"
    }));
}
