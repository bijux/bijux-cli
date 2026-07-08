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

fn write_success_graph(path: &Path) {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"summary-success","owners":[],"tags":[]},
            "nodes":[
                {
                    "id":"seed",
                    "kind":"const",
                    "outputs":[{"name":"value","path":"seed/value.json"}],
                    "params":{"value":"alpha"}
                },
                {
                    "id":"echo",
                    "kind":"const",
                    "outputs":[{"name":"value","path":"echo/value.json"}],
                    "params":{"value":"beta"}
                }
            ],
            "edges":[]
        }))
        .expect("graph json"),
    )
    .expect("write graph");
}

fn write_failure_graph(path: &Path) {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"summary-failure","owners":[],"tags":[]},
            "nodes":[
                {
                    "id":"explode",
                    "kind":"shell",
                    "outputs":[{"name":"out","path":"explode/out.txt"}],
                    "effects":["filesystem"],
                    "params":{"argv":["/bin/sh","-c","exit 7"]}
                }
            ],
            "edges":[]
        }))
        .expect("graph json"),
    )
    .expect("write graph");
}

#[test]
fn run_json_includes_operator_completion_summary_for_successful_runs() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    let out_dir = temp.path().join("runs");
    write_success_graph(&graph);
    fs::create_dir_all(&out_dir).expect("runs dir");

    let payload = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );

    let summary = &payload["data"]["summary"];
    assert_eq!(summary["status"], "success");
    assert!(summary["duration_ms"].is_number());
    assert_eq!(summary["node_counts"]["success"], 2);
    assert_eq!(summary["cache_hits"], 0);
    assert_eq!(summary["artifact_count"], 2);
    assert_eq!(summary["promoted_artifact_count"], 0);
    assert_eq!(summary["suggested_next_action"]["action"], "inspect-run");
    assert!(summary["suggested_next_action"]["command"]
        .as_str()
        .is_some_and(|command| command.contains("bijux-dag runs inspect")));
}

#[test]
fn run_human_output_prints_compact_completion_summary() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    let out_dir = temp.path().join("runs");
    write_success_graph(&graph);
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = run_dag_command(
        &["run", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("run_summary_status: \"success\""));
    assert!(stdout.contains("run_summary_node_counts:"));
    assert!(stdout.contains("\"success\":2"));
    assert!(stdout.contains("run_summary_next_command: \"bijux-dag runs inspect"));
}

#[test]
fn run_json_failure_surface_preserves_completion_summary_when_run_dir_exists() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    let out_dir = temp.path().join("runs");
    write_failure_graph(&graph);
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = run_dag_command(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    assert_eq!(code, 0, "stderr={stderr}");
    let payload: Value = serde_json::from_str(&stdout).expect("parse json envelope");
    let summary = &payload["data"]["summary"];
    assert_eq!(summary["status"], "failed");
    assert_eq!(summary["suggested_next_action"]["action"], "explain-failure");
    assert!(summary["failed_node_reasons"].as_array().is_some_and(|reasons| !reasons.is_empty()));
}
