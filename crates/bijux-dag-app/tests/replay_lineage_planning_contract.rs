use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tar as _;
use tempfile as _;
use thiserror as _;

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0, "command failed: code={code} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run_dir"))
}

fn read_manifest(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
        .expect("manifest json")
}

#[test]
fn replay_manifest_keeps_parent_run_ancestry_chain() {
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
            "ancestry-source",
        ],
        &root,
    );
    let source_run = run_dir_from_response(&run);
    let replay = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "ancestry-replay",
        ],
        &root,
    );
    let replay_run = run_dir_from_response(&replay);
    let source_manifest = read_manifest(&source_run);
    let replay_manifest = read_manifest(&replay_run);
    assert_eq!(replay_manifest["run_metadata"]["parent_run_id"], source_manifest["run_id"]);
}

#[test]
fn replay_accepts_imported_run_as_source() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let bundle = tmp.path().join("bundle.json");

    let source = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "import-source",
        ],
        &root,
    );
    let source_run = run_dir_from_response(&source);
    let _ = run_json(
        &[
            "export",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&bundle),
            "--manifest-only",
        ],
        &root,
    );
    let imported = run_json(&["import", "--json", &output_path_string(&bundle)], &root);
    assert_eq!(imported["ok"], true);
    let replay = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "imported-replay",
            "--prove",
        ],
        &root,
    );
    assert_eq!(replay["ok"], true);
    assert!(replay["data"]["replay_proof"].is_object());
}

#[test]
fn replay_partial_selection_emits_dry_run_plan_with_selectors() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("crates/bijux-dag-core/tests/snapshots/selective_replay.dag.json");

    let source = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "partial-source",
        ],
        &root,
    );
    let source_run = run_dir_from_response(&source);
    let dry = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&out_dir),
            "--dry-run",
            "--select",
            "id:replay_check",
        ],
        &root,
    );
    assert!(dry["data"]["dry_run_plan"].is_object());
    assert!(dry["data"]["dry_run_plan"]["selectors"]["select"]
        .as_array()
        .is_some_and(|v| v.iter().any(|entry| entry == "id:replay_check")));
}

#[test]
fn replay_accepts_source_run_id_with_explicit_run_root() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");

    let _source = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "source-by-id",
        ],
        &root,
    );

    let replay = run_json(
        &[
            "replay",
            "--json",
            "--source-run-id",
            "source-by-id",
            "--source-run-root",
            &output_path_string(&out_dir),
            "--out",
            &output_path_string(&out_dir),
            "--dry-run",
        ],
        &root,
    );
    assert_eq!(replay["ok"], true);
    assert_eq!(replay["data"]["dry_run_plan"]["source_run_id"], "source-by-id");
}

#[test]
fn replay_rejects_corrupt_upstream_artifact_at_rerun_boundary() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("crates/bijux-dag-core/tests/snapshots/selective_replay.dag.json");

    let source = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "boundary-source",
        ],
        &root,
    );
    let source_run = run_dir_from_response(&source);
    fs::write(source_run.join("nodes/source/outputs/source/out"), "corrupt")
        .expect("corrupt source");

    let (code, stdout, stderr) = run_dag(
        &[
            "replay",
            "--json",
            "--source-run-id",
            "boundary-source",
            "--source-run-root",
            &output_path_string(&out_dir),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "boundary-replay",
            "--from-node",
            "branch_a",
        ],
        &root,
    );
    assert_eq!(code, 3, "stdout={stdout} stderr={stderr}");
    let payload: Value = serde_json::from_str(&stdout).expect("parse replay rejection");
    assert_eq!(payload["ok"], false);
    assert_eq!(
        payload["data"]["message"],
        "upstream artifact verification failed for the requested replay boundary"
    );
    assert_eq!(payload["data"]["upstream_artifact_verification"]["verified"], false);
    assert!(payload["data"]["upstream_artifact_verification"]["checks"]
        .as_array()
        .is_some_and(|checks| checks.iter().any(|check| check["verified"] == false)));
}

#[test]
fn replay_reports_node_scoped_diff_for_single_rerun_root() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = root.join("crates/bijux-dag-core/tests/snapshots/selective_replay.dag.json");

    let source = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "node-diff-source",
        ],
        &root,
    );
    let source_run = run_dir_from_response(&source);
    let trace_path = source_run.join("nodes/branch_a/trace.json");
    let mut trace: Value =
        serde_json::from_str(&fs::read_to_string(&trace_path).expect("read trace"))
            .expect("parse trace");
    trace["status"] = Value::String("failed".to_string());
    fs::write(&trace_path, serde_json::to_vec_pretty(&trace).expect("encode trace"))
        .expect("write trace");

    let replay = run_json(
        &[
            "replay",
            "--json",
            "--source-run-id",
            "node-diff-source",
            "--source-run-root",
            &output_path_string(&out_dir),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "node-diff-replay",
            "--from-node",
            "branch_a",
        ],
        &root,
    );
    assert_eq!(replay["ok"], true);
    assert_eq!(replay["data"]["node_rerun_diff"]["node_id"], "branch_a");
    assert_eq!(replay["data"]["node_rerun_diff"]["summary"]["node"], "branch_a");
    assert_eq!(replay["data"]["node_rerun_diff"]["summary"]["equivalent"], false);
    assert!(replay["data"]["node_rerun_diff"]["causal_chain"]
        .as_array()
        .is_some_and(|chain| chain.len() > 1));
}
