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
use serde_json::{json, Value};
use sha2 as _;
use std::process::Command;
use std::sync::OnceLock;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_dag_command(args: &[&str], cwd: &Path) -> (i32, String, String) {
    let output = Command::new(resolve_bijux_dag_binary())
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run dag command");
    (
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn resolve_bijux_dag_binary() -> PathBuf {
    static BIN_PATH: OnceLock<PathBuf> = OnceLock::new();
    BIN_PATH
        .get_or_init(|| {
            if let Some(path) = std::env::var_os("BIJUX_DAG_BIN").map(PathBuf::from) {
                if path.exists() {
                    return path;
                }
            }
            let workspace_root = repo_root();
            let target_root = workspace_root.join("artifacts").join("test-bin-target");
            let status = Command::new("cargo")
                .current_dir(&workspace_root)
                .env("RUSTFLAGS", "-Awarnings")
                .env("CARGO_TARGET_DIR", &target_root)
                .args(["build", "-q", "-p", "bijux-dag-cli"])
                .status()
                .expect("build bijux-dag binary");
            assert!(status.success(), "failed to build bijux-dag binary");
            target_root.join("debug").join(format!("bijux-dag{}", std::env::consts::EXE_SUFFIX))
        })
        .clone()
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag_command(args, cwd);
    assert_eq!(code, 0, "command failed: stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn write_history_run(
    run_dir: &Path,
    run_id: &str,
    status: &str,
    created_unix_ms: u64,
    graph_name: &str,
    graph_fingerprint: &str,
    parent_run_id: Option<&str>,
    finished_unix_ms: Option<u64>,
    lifecycle_active: bool,
) {
    fs::create_dir_all(run_dir.join("outputs")).expect("outputs dir");
    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id": run_id,
            "created_unix_ms": created_unix_ms,
            "started_unix_ms": created_unix_ms + 1,
            "finished_unix_ms": finished_unix_ms,
            "graph_snapshot":"graph.snapshot.json",
            "status": status,
            "spec":"bijux-dag/v0.1",
            "graph_fingerprint": graph_fingerprint,
            "tool_version":"0.4.0",
            "jobs":1,
            "adapters":[],
            "outputs":[],
            "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0,"cancelled":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true},
            "run_metadata":{
                "submission_source":"manual",
                "trigger_source":"cli",
                "operator":"tester",
                "labels":["status"],
                "parent_run_id": parent_run_id,
                "source_run_id": parent_run_id
            }
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "graph_fingerprint": graph_fingerprint,
            "graph": {
                "meta": {"name": graph_name},
                "nodes": [{"id":"n1"}],
                "edges": []
            }
        }))
        .expect("graph snapshot"),
    )
    .expect("write graph snapshot");
    fs::write(run_dir.join("snapshot.json"), "{}").expect("write legacy snapshot");
    fs::write(run_dir.join("outputs").join("index.json"), "{\"files\":[]}").expect("write outputs");
    fs::write(run_dir.join("outputs.index.json"), "{\"files\":[]}").expect("write legacy outputs");
    if lifecycle_active {
        fs::write(
            run_dir.join(".run-incomplete.json"),
            r#"{"reason":"run not finalized; recover or repair before treating artifacts as complete"}"#,
        )
        .expect("write incomplete marker");
    }
}

#[test]
fn runs_history_json_filters_by_graph_and_surfaces_lineage_output_and_lifecycle() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let runs_root = temp.path().join("runs");

    write_history_run(
        &runs_root.join("run-parent"),
        "run-parent",
        "success",
        10,
        "training-pipeline",
        "graph-train",
        None,
        Some(12),
        false,
    );
    write_history_run(
        &runs_root.join("run-child"),
        "run-child",
        "running",
        20,
        "training-pipeline",
        "graph-train",
        Some("run-parent"),
        None,
        true,
    );
    write_history_run(
        &runs_root.join("run-other"),
        "run-other",
        "success",
        30,
        "reporting-pipeline",
        "graph-report",
        None,
        Some(32),
        false,
    );

    let payload = run_json(
        &[
            "runs",
            "history",
            "--json",
            "--root",
            &output_path_string(&runs_root),
            "--graph",
            "training-pipeline",
            "--status",
            "running",
        ],
        &root,
    );

    let rows = payload["data"]["runs"].as_array().expect("history rows");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row["run_id"], "run-child");
    assert_eq!(row["lifecycle_state"], "active");
    assert_eq!(row["graph_name"], "training-pipeline");
    assert_eq!(row["graph_fingerprint"], "graph-train");
    assert_eq!(row["run_dir"], "run-child");
    assert_eq!(row["output_location"], "run-child/outputs");
    assert_eq!(row["lineage"]["parent_run_id"], "run-parent");
}

#[test]
fn runs_history_json_orders_recent_runs_and_surfaces_child_lineage() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let runs_root = temp.path().join("runs");

    write_history_run(
        &runs_root.join("run-parent"),
        "run-parent",
        "success",
        10,
        "training-pipeline",
        "graph-train",
        None,
        Some(12),
        false,
    );
    write_history_run(
        &runs_root.join("run-child"),
        "run-child",
        "success",
        20,
        "training-pipeline",
        "graph-train",
        Some("run-parent"),
        Some(22),
        false,
    );

    let payload =
        run_json(&["runs", "history", "--json", "--root", &output_path_string(&runs_root)], &root);

    let rows = payload["data"]["runs"].as_array().expect("history rows");
    assert_eq!(rows[0]["run_id"], "run-child");
    assert_eq!(rows[1]["run_id"], "run-parent");
    assert_eq!(rows[1]["lineage"]["child_run_ids"], json!(["run-child"]));
    assert_eq!(rows[1]["lifecycle_state"], "historical");
}

#[test]
fn runs_history_human_output_reports_status_graph_and_output_location() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let runs_root = temp.path().join("runs");

    write_history_run(
        &runs_root.join("run-active"),
        "run-active",
        "running",
        50,
        "training-pipeline",
        "graph-train",
        None,
        None,
        true,
    );

    let (code, stdout, stderr) = run_dag_command(
        &[
            "runs",
            "history",
            "--root",
            &output_path_string(&runs_root),
            "--graph",
            "training-pipeline",
        ],
        &root,
    );
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("run-active status=running lifecycle=active"));
    assert!(stdout.contains("graph=training-pipeline"));
    assert!(stdout.contains("output="));
}
