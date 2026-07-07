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

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

#[test]
#[ignore = "experimental"]
fn hash_graph_cli_output_matches_core_graph_id() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let dag_path = tmp.path().join("graph.dag.json");
    let payload = r#"{
  "spec": "bijux-dag/v0.1",
  "nodes": [
    {
      "id": "const1",
      "kind": "const",
      "outputs": [{"name": "out", "path": "const1/out.txt"}],
      "params": {"value": "hello"}
    }
  ],
  "edges": []
}"#;
    fs::write(&dag_path, payload).expect("write dag");

    let graph = bijux_dag_core::parse_graph_strict(payload).expect("parse core graph");
    let core_id = graph.graph_id().expect("core graph id").as_str().to_string();

    let dag_path_str = dag_path.to_string_lossy().into_owned();
    let args = ["hash", "graph", "--json", dag_path_str.as_str()];
    let (code, stdout, stderr) = run_dag(&args, &root);
    assert!(code == 0, "hash graph failed: stderr={stderr}");
    let value: Value = serde_json::from_str(&stdout).expect("parse cli json");
    assert_eq!(value["command"], "dag.hash.graph");
    let cli_hash = value["data"]["graph_id"].as_str().expect("cli hash");
    assert_eq!(core_id, cli_hash);
}
