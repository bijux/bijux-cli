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
use tar as _;
use tempfile as _;
use thiserror as _;

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert_eq!(code, 0, "command failed: {stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn extract_run_dir(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run_dir string"))
}

#[test]
fn prove_reports_complete_bundle_for_valid_run() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);

    let proof = run_json(&["prove", "--json", &output_path_string(&run_dir)], &root);
    assert_eq!(proof["command"], "dag.prove");
    assert_eq!(proof["ok"], true);
    assert_eq!(proof["data"]["schema_version"], "proof-bundle/v0.1");
    assert_eq!(proof["data"]["complete"], true);
}

#[test]
fn prove_reports_incomplete_for_corrupt_run() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    fs::remove_file(run_dir.join("outputs").join("index.json")).expect("remove outputs index");

    let (code, stdout, _stderr) =
        run_dag(&["prove", "--json", &output_path_string(&run_dir)], &root);
    assert_ne!(code, 0, "corrupt run proof should be incomplete and non-zero");
    let proof: Value = serde_json::from_str(&stdout).expect("parse proof payload");
    assert_eq!(proof["command"], "dag.prove");
    assert_eq!(proof["ok"], false);
    assert_eq!(proof["data"]["complete"], false);
    assert!(proof["data"]["incomplete_reasons"].is_array());
}

#[test]
#[ignore = "experimental"]
fn prove_reports_backend_origin_and_unsigned_trust_boundary() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    fs::write(
        run_dir.join("provenance.json"),
        r#"{"source":"remote-run","source_run_dir":"C:\\legacy\\run-123"}"#,
    )
    .expect("write provenance");

    let proof = run_json(&["prove", "--json", &output_path_string(&run_dir)], &root);
    assert_eq!(proof["data"]["backend_origin"], "remote-run");
    assert_eq!(proof["data"]["signing"]["signed"], false);
    assert_eq!(proof["data"]["signing"]["trust_level"], "unsigned");
}

#[test]
fn prove_reports_incomplete_for_hash_corruption() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create runs");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let run = run_json(
        &["run", "--json", &output_path_string(&graph), "--out", &output_path_string(&out_dir)],
        &root,
    );
    let run_dir = extract_run_dir(&run);
    let outputs_index = run_dir.join("outputs").join("index.json");
    let mut index: Value =
        serde_json::from_str(&fs::read_to_string(&outputs_index).expect("read outputs"))
            .expect("parse outputs");
    if let Some(first) =
        index.get_mut("files").and_then(Value::as_array_mut).and_then(|v| v.first_mut())
    {
        first["sha256"] = Value::String("deadbeef".to_string());
    }
    fs::write(&outputs_index, serde_json::to_vec_pretty(&index).expect("encode outputs"))
        .expect("write outputs");

    let (code, stdout, _stderr) =
        run_dag(&["prove", "--json", &output_path_string(&run_dir)], &root);
    assert_ne!(code, 0);
    let proof: Value = serde_json::from_str(&stdout).expect("parse proof");
    assert_eq!(proof["data"]["complete"], false);
}
