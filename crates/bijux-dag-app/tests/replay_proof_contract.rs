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
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar as _;
use tempfile as _;
use thiserror as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    let output = Command::new("cargo")
        .current_dir(cwd)
        .env("RUSTFLAGS", "-Awarnings")
        .env("CARGO_TARGET_DIR", cwd.join("artifacts/target"))
        .args(["run", "-p", "bijux-dag-cli", "--", "dag"])
        .args(args)
        .output()
        .expect("run dag command");
    (
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0, "command failed: code={code} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[test]
fn replay_dry_run_and_prove_surfaces_are_machine_readable() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "source-proof",
        ],
        &root,
    );
    assert_eq!(run["ok"], true);

    let source_run = out_dir.join("run-source-proof");
    let dry = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&out_dir),
            "--dry-run",
            "--prove",
        ],
        &root,
    );
    assert!(dry["data"]["dry_run_plan"].is_object());
    assert!(dry["data"]["run_dir"].is_null());

    let proved = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "replay-proof",
            "--prove",
        ],
        &root,
    );
    assert_eq!(proved["ok"], true);
    assert!(proved["data"]["run_dir"].is_string());
    assert!(proved["data"]["replay_proof"].is_object());
    assert!(proved["data"]["replay_proof"]["fidelity_level"].is_string());
    assert!(proved["data"]["replay_proof"]["safety_level"].is_string());
    assert!(proved["data"]["replay_proof"]["branch_decision_drift_nodes"].is_array());
    assert!(proved["data"]["replay_proof"]["source_evidence_gaps"].is_array());
    assert!(proved["data"]["replay_proof"]["replay_evidence_gaps"].is_array());
}

#[test]
fn replay_prove_reports_strict_equivalent_on_exact_pair() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let _ = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "exact-source",
        ],
        &root,
    );

    let source_run = out_dir.join("run-exact-source");
    let proved = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "exact-replay",
            "--prove",
        ],
        &root,
    );
    assert_eq!(proved["ok"], true);
    assert_eq!(proved["data"]["replay_proof"]["fidelity_level"], "strict_equivalent");
    assert_eq!(proved["data"]["replay_proof"]["safety_level"], "equivalent");
}

#[test]
fn replay_prove_reports_diverged_on_corrupt_source_pair() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let _ = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "drift-source",
        ],
        &root,
    );

    let source_run = out_dir.join("run-drift-source");
    let first_node_dir = std::fs::read_dir(source_run.join("nodes"))
        .expect("read nodes dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .expect("at least one node dir");
    let trace_path = first_node_dir.join("trace.json");
    let mut trace: Value =
        serde_json::from_str(&fs::read_to_string(&trace_path).expect("read trace"))
            .expect("parse trace");
    trace["status"] = Value::String("failed".to_string());
    fs::write(&trace_path, serde_json::to_vec_pretty(&trace).expect("encode trace"))
        .expect("write trace");

    let proved = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "drift-replay",
            "--prove",
        ],
        &root,
    );
    assert_eq!(proved["ok"], true);
    assert_eq!(proved["data"]["replay_proof"]["fidelity_level"], "diverged");
    assert_eq!(proved["data"]["replay_proof"]["safety_level"], "risky");
}
