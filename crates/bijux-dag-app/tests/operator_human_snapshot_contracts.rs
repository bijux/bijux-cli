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
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar as _;
use tempfile as _;
use thiserror as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn dag_command(root: &Path) -> Command {
    if let Some(path) = std::env::var_os("BIJUX_DAG_BIN") {
        let path = PathBuf::from(path);
        if path.exists() {
            let mut command = Command::new(path);
            command.current_dir(root);
            return command;
        }
    }
    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let mut command = Command::new(cargo_bin);
    command.current_dir(root).env("CARGO_TARGET_DIR", root.join("artifacts/target")).args([
        "run",
        "--quiet",
        "-p",
        "bijux-dag-cli",
        "--",
    ]);
    command
}

fn run_human_with_code(root: &Path, expected_code: i32, args: &[&str]) -> String {
    let output = dag_command(root).args(args).output().expect("run dag command");
    assert_eq!(
        output.status.code().unwrap_or(1),
        expected_code,
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_human(root: &Path, args: &[&str]) -> String {
    run_human_with_code(root, 0, args)
}

fn write_run_with_fixed_id(root: &Path, graph: &Path, out_dir: &Path, run_id: &str) -> PathBuf {
    fs::create_dir_all(out_dir).expect("create run output root");
    let graph_arg = graph.to_string_lossy().to_string();
    let out_arg = out_dir.to_string_lossy().to_string();
    let output = run_human(
        root,
        &["run", graph_arg.as_str(), "--out", out_arg.as_str(), "--run-id", run_id],
    );
    let run_dir = output
        .lines()
        .find_map(|line| line.strip_prefix("run dir: "))
        .expect("run output includes run dir");
    PathBuf::from(run_dir)
}

fn normalize_paths(text: &str, tmp_root: &Path) -> String {
    text.replace(&*tmp_root.to_string_lossy(), "<TMP>")
}

fn normalize_run_human_output(text: &str, tmp_root: &Path) -> String {
    normalize_paths(text, tmp_root)
        .lines()
        .map(|line| {
            if line.starts_with("run_summary_duration_ms: ") {
                "run_summary_duration_ms: <DURATION_MS>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn validate_human_output_snapshot_is_stable() {
    let root = repo_root();
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let out = run_human(&root, &["validate", graph.to_string_lossy().as_ref()]);
    assert_eq!(out, include_str!("snapshots/validate_human_output.txt"),);
}

#[test]
fn plan_human_output_snapshot_is_stable() {
    let root = repo_root();
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let out = run_human(&root, &["plan", "explain", graph.to_string_lossy().as_ref()]);
    assert_eq!(out, include_str!("snapshots/plan_human_output.txt"));
}

#[test]
fn run_human_output_snapshot_is_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let out = run_human(
        &root,
        &[
            "run",
            graph.to_string_lossy().as_ref(),
            "--out",
            tmp.path().join("runs").to_string_lossy().as_ref(),
            "--run-id",
            "run-fixed",
        ],
    );
    let normalized = normalize_run_human_output(&out, tmp.path());
    assert_eq!(normalized, include_str!("snapshots/run_human_output.txt"));
}

#[test]
fn inspect_human_output_snapshot_is_stable() {
    let summary = serde_json::json!({
        "run_id":"run-fixed",
        "status":"success",
        "origin":"native",
        "integrity_state":"healthy",
        "retry_count":0,
        "cache_hits":0,
        "artifact_count":0,
        "timing_ms":{"started":1000u64,"finished":1010u64}
    });
    let text = bijux_dag_app::format_inspect_human(&summary);
    assert_eq!(text.trim_end(), include_str!("snapshots/inspect_human_output.txt").trim_end());
}

#[test]
fn history_human_output_snapshot_is_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let out_dir = tmp.path().join("runs");
    write_run_with_fixed_id(&root, &graph, &out_dir, "run-fixed");
    let history =
        run_human(&root, &["runs", "history", "--root", out_dir.to_string_lossy().as_ref()]);
    assert_eq!(history, include_str!("snapshots/history_human_output.txt"));
}

#[test]
fn replay_human_output_snapshot_is_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let out_dir = tmp.path().join("runs");
    let run_dir = write_run_with_fixed_id(&root, &graph, &out_dir, "run-fixed");
    let replay = run_human(
        &root,
        &[
            "replay",
            run_dir.to_string_lossy().as_ref(),
            "--out",
            out_dir.to_string_lossy().as_ref(),
            "--run-id",
            "replay-fixed",
            "--dry-run",
        ],
    );
    let normalized = normalize_paths(&replay, tmp.path());
    assert_eq!(normalized, include_str!("snapshots/replay_human_output_contract.txt"));
}

#[test]
fn diff_human_output_snapshot_is_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let out_dir = tmp.path().join("runs");
    let run_dir = write_run_with_fixed_id(&root, &graph, &out_dir, "run-fixed");
    let diff = run_human(
        &root,
        &[
            "diff",
            run_dir.to_string_lossy().as_ref(),
            run_dir.to_string_lossy().as_ref(),
            "--explain",
        ],
    );
    assert_eq!(diff, include_str!("snapshots/diff_human_output_contract.txt"));
}

#[test]
#[ignore = "experimental"]
fn prove_human_output_snapshot_is_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let out_dir = tmp.path().join("runs");
    let run_dir = write_run_with_fixed_id(&root, &graph, &out_dir, "run-fixed");
    let prove = run_human_with_code(&root, 0, &["prove", run_dir.to_string_lossy().as_ref()]);
    assert_eq!(prove, include_str!("snapshots/prove_human_output_contract.txt"));
}

#[test]
fn verify_human_output_snapshot_is_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let out_dir = tmp.path().join("runs");
    let run_dir = write_run_with_fixed_id(&root, &graph, &out_dir, "run-fixed");
    let verify = run_human_with_code(&root, 0, &["verify", run_dir.to_string_lossy().as_ref()]);
    assert_eq!(verify, include_str!("snapshots/verify_human_output_contract.txt"));
}

#[test]
fn artifact_inspect_human_output_snapshot_is_stable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph = root.join("evidence/authoring/examples/multi-output-artifact.dag.json");
    let out_dir = tmp.path().join("runs");
    let run_dir = write_run_with_fixed_id(&root, &graph, &out_dir, "art-fixed");
    let outputs: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("outputs/index.json")).expect("read outputs index"),
    )
    .expect("parse outputs");
    let first = &outputs["files"][0];
    let artifact_id = format!(
        "{}:{}",
        first["node_id"].as_str().expect("node id"),
        first["path"].as_str().expect("path").rsplit('/').next().expect("file name")
    );
    let out = run_human(
        &root,
        &["artifact-inspect", run_dir.to_string_lossy().as_ref(), artifact_id.as_str()],
    );
    assert_eq!(out, include_str!("snapshots/artifact_inspect_human_output.txt"));
}
