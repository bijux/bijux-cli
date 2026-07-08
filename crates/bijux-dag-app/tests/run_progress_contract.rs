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
use serde_json::json;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tar as _;
use tempfile as _;
use thiserror as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
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

fn parse_json_stream(stdout: &str) -> Vec<Value> {
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).expect("parse json line"))
        .collect()
}

fn write_long_running_graph(path: &Path) {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"progress-live","owners":[],"tags":[]},
            "nodes":[
                {
                    "id":"wait",
                    "kind":"shell",
                    "outputs":[{"name":"out","path":"out.txt"}],
                    "effects":["filesystem"],
                    "params":{"argv":["/bin/sh","-c","sleep 1; printf ok > ../outputs/out.txt"]}
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
            "meta":{"name":"progress-failure","owners":[],"tags":[]},
            "nodes":[
                {
                    "id":"explode",
                    "kind":"shell",
                    "outputs":[{"name":"out","path":"explode/out.txt"}],
                    "effects":["filesystem"],
                    "params":{"argv":["/bin/sh","-c","sleep 0.1; exit 7"]}
                }
            ],
            "edges":[]
        }))
        .expect("graph json"),
    )
    .expect("write graph");
}

#[test]
fn run_compact_progress_falls_back_to_noninteractive_status_lines() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    let out_dir = temp.path().join("runs");
    write_long_running_graph(&graph);
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = run_dag_command(
        &[
            "run",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--progress",
            "compact",
        ],
        &root,
    );

    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stderr.contains("progress elapsed="));
    assert!(stderr.contains("active=[wait]"), "stderr={stderr}");
    assert!(stdout.contains("run_summary_status: \"success\""));
}

#[test]
fn run_compact_progress_reports_latest_failure_on_error() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    let out_dir = temp.path().join("runs");
    write_failure_graph(&graph);
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, _stdout, stderr) = run_dag_command(
        &[
            "run",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--progress",
            "compact",
        ],
        &root,
    );

    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stderr.contains("progress elapsed="), "stderr={stderr}");
    assert!(stderr.contains("latest_failure=explode:failed"), "stderr={stderr}");
}

#[test]
fn run_json_progress_streams_snapshots_and_final_result() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    let out_dir = temp.path().join("runs");
    write_long_running_graph(&graph);
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = run_dag_command(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--progress",
            "compact",
        ],
        &root,
    );

    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stderr.is_empty(), "json progress should stay off stderr: {stderr}");

    let events = parse_json_stream(&stdout);
    assert!(
        events.iter().any(|event| event["command"] == "dag.run.progress"),
        "expected at least one progress event: {stdout}"
    );
    assert_eq!(events.last().expect("final event")["command"], "dag.run");
    assert!(events.last().expect("final event")["data"]["summary"].is_object());
}

#[test]
fn run_json_progress_surfaces_failures_as_they_happen() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("graph.json");
    let out_dir = temp.path().join("runs");
    write_failure_graph(&graph);
    fs::create_dir_all(&out_dir).expect("runs dir");

    let (code, stdout, stderr) = run_dag_command(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--progress",
            "compact",
        ],
        &root,
    );

    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stderr.is_empty(), "json progress should stay off stderr: {stderr}");

    let events = parse_json_stream(&stdout);
    let failure_progress = events
        .iter()
        .find(|event| {
            event["command"] == "dag.run.progress"
                && event["data"]["snapshot"]["latest_failure"]["node_id"] == "explode"
        })
        .expect("failure progress event");
    assert_eq!(failure_progress["data"]["snapshot"]["latest_failure"]["status"], "failed");
    assert_eq!(events.last().expect("final event")["command"], "dag.run");
    assert_eq!(events.last().expect("final event")["data"]["summary"]["status"], "failed");
}
