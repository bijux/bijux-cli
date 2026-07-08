use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{dag_command, dag_run};

#[test]
fn cli_validate_smoke_accepts_valid_graph() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let graph_path = tmp.path().join("valid.graph.json");
    let graph = serde_json::json!({
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"validate-smoke","owners":[],"tags":[]},
      "nodes":[
        {
          "id":"n1",
          "kind":"const",
          "inputs":[],
          "outputs":[{"name":"out","path":"n1/out.txt"}],
          "effects":["filesystem"],
          "params":{"value":1}
        }
      ],
      "edges":[]
    });
    std::fs::write(&graph_path, serde_json::to_vec_pretty(&graph).expect("encode"))
        .expect("write graph");

    let matches = dag_command()
        .try_get_matches_from(["bijux-dag", "validate", graph_path.to_string_lossy().as_ref()])
        .expect("cli parse");
    let code = dag_run(&matches).expect("validate command");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn cli_validate_smoke_reports_invalid_graph() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let graph_path = tmp.path().join("invalid.graph.json");
    std::fs::write(&graph_path, b"{ not-valid-json").expect("write graph");

    let matches = dag_command()
        .try_get_matches_from(["bijux-dag", "validate", graph_path.to_string_lossy().as_ref()])
        .expect("cli parse");
    let err = dag_run(&matches).expect_err("invalid graph should fail validation");
    assert_eq!(err, std::process::ExitCode::from(2));
}
