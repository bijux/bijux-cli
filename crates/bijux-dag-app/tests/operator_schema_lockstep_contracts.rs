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

fn run_json_with_code(root: &Path, expected_code: i32, args: &[&str]) -> serde_json::Value {
    let output = dag_command(root).args(args).output().expect("run dag command");
    assert_eq!(
        output.status.code().unwrap_or(1),
        expected_code,
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse json envelope")
}

fn run_json(root: &Path, args: &[&str]) -> serde_json::Value {
    run_json_with_code(root, 0, args)
}

fn run_json_with_internal_lane(root: &Path, args: &[&str]) -> serde_json::Value {
    let output = dag_command(root)
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(args)
        .output()
        .expect("run dag command");
    assert_eq!(
        output.status.code().unwrap_or(1),
        0,
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse json envelope")
}

fn required_fields(schema_rel: &str) -> Vec<String> {
    let root = repo_root();
    let schema: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(schema_rel)).expect("read schema"))
            .expect("parse schema");
    schema["required"]
        .as_array()
        .expect("required array")
        .iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect()
}

#[test]
#[ignore = "slow"]
fn capability_query_output_schema_lockstep() {
    let root = repo_root();
    let payload =
        run_json_with_internal_lane(&root, &["--json", "capabilities", "--backend", "hpc"]);
    let data = payload["data"].as_object().expect("capability data object");
    for field in required_fields("configs/dag/schema/operator/capability_query.schema.json") {
        assert!(data.contains_key(&field), "capability output missing required field: {field}");
    }
}

#[test]
fn verify_output_schema_lockstep() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create out dir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    run_json(
        &root,
        &[
            "--json",
            "run",
            graph.to_string_lossy().as_ref(),
            "--out",
            out_dir.to_string_lossy().as_ref(),
            "--run-id",
            "run-fixed",
        ],
    );
    let verify = run_json_with_code(
        &root,
        0,
        &["--json", "verify", out_dir.join("run-fixed").to_string_lossy().as_ref()],
    );
    let data = verify["data"].as_object().expect("verify data");
    for field in required_fields("configs/dag/schema/operator/verify_output.schema.json") {
        assert!(data.contains_key(&field), "verify missing required field: {field}");
    }
}

#[test]
#[ignore = "slow"]
fn prove_output_schema_lockstep() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create out dir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    run_json(
        &root,
        &[
            "--json",
            "run",
            graph.to_string_lossy().as_ref(),
            "--out",
            out_dir.to_string_lossy().as_ref(),
            "--run-id",
            "run-fixed",
        ],
    );
    let prove = run_json_with_code(
        &root,
        0,
        &["--json", "prove", out_dir.join("run-fixed").to_string_lossy().as_ref()],
    );
    let data = prove["data"].as_object().expect("prove data");
    for field in required_fields("configs/dag/schema/operator/prove_output.schema.json") {
        assert!(data.contains_key(&field), "prove missing required field: {field}");
    }
}

#[test]
#[ignore = "slow"]
fn export_summary_schema_lockstep() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("create out dir");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    run_json(
        &root,
        &[
            "--json",
            "run",
            graph.to_string_lossy().as_ref(),
            "--out",
            out_dir.to_string_lossy().as_ref(),
            "--run-id",
            "run-fixed",
        ],
    );
    let bundle = tmp.path().join("bundle.json");
    let export = run_json(
        &root,
        &[
            "--json",
            "export",
            out_dir.join("run-fixed").to_string_lossy().as_ref(),
            "--out",
            bundle.to_string_lossy().as_ref(),
            "--manifest-only",
        ],
    );
    let data = export["data"].as_object().expect("export data");
    for field in required_fields("configs/dag/schema/operator/export_summary.schema.json") {
        assert!(data.contains_key(&field), "export missing required field: {field}");
    }
}

#[test]
#[ignore = "slow"]
fn import_summary_schema_lockstep() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle = tmp.path().join("bundle.json");
    fs::write(
        &bundle,
        r#"{"bundle_version":"export-bundle/v0.1","export_mode":"manifest-only","manifest":{},"graph_snapshot":{},"outputs":{},"node_traces":{},"provenance":{"source":"native-run","lineage":[],"source_run_id":"r1"}}"#,
    )
    .expect("write bundle");
    let import =
        run_json(&root, &["--json", "import", bundle.to_string_lossy().as_ref(), "--verify-only"]);
    let data = import["data"].as_object().expect("import data");
    for field in required_fields("configs/dag/schema/operator/import_summary.schema.json") {
        assert!(data.contains_key(&field), "import missing required field: {field}");
    }
}
