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

use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn write_graph(path: &Path, payload: serde_json::Value) {
    fs::write(path, serde_json::to_vec_pretty(&payload).expect("graph json")).expect("write graph");
}

#[test]
fn validate_json_reports_missing_required_input_binding() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("missing-input.json");
    write_graph(
        &graph,
        serde_json::json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"missing-input-binding","owners":[],"tags":[]},
            "nodes":[
                {
                    "id":"emit",
                    "kind":"const",
                    "outputs":[{"name":"value","path":"emit/value.json"}],
                    "params":{"value":"seed"}
                },
                {
                    "id":"consume",
                    "kind":"const",
                    "inputs":["payload"],
                    "outputs":[{"name":"result","path":"consume/result.json"}],
                    "params":{"value":1}
                }
            ],
            "edges":[]
        }),
    );

    let (code, stdout, stderr) =
        support::run_dag_command(&["validate", "--json", &output_path_string(&graph)], &root);

    assert_eq!(code, 2, "stderr={stderr}");
    assert!(stderr.is_empty(), "stderr should stay empty for json output: {stderr}");
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], false);
    assert!(payload["diagnostics"].as_array().is_some());
    assert!(payload["diagnostics"].as_array().unwrap().iter().any(|diagnostic| {
        diagnostic["code"] == "E1005"
            && diagnostic["message"] == "missing required input binding: consume.payload"
    }));
}

#[test]
fn validate_json_reports_invalid_container_workdir_path() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph = temp.path().join("invalid-workdir.json");
    write_graph(
        &graph,
        serde_json::json!({
            "spec":"bijux-dag/v0.1",
            "meta":{"name":"invalid-container-workdir","owners":[],"tags":[]},
            "nodes":[
                {
                    "id":"publish",
                    "kind":"container",
                    "outputs":[{"name":"result","path":"publish/result.txt"}],
                    "container":{
                        "image":"alpine:3.20",
                        "argv":["sh","-c","echo ok > result.txt"],
                        "engine":"docker",
                        "workdir":"{work_dir}/../escape"
                    },
                    "effects":["filesystem"]
                }
            ],
            "edges":[]
        }),
    );

    let (code, stdout, stderr) =
        support::run_dag_command(&["validate", "--json", &output_path_string(&graph)], &root);

    assert_eq!(code, 2, "stderr={stderr}");
    assert!(stderr.is_empty(), "stderr should stay empty for json output: {stderr}");
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("validate json");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], false);
    assert!(payload["diagnostics"].as_array().unwrap().iter().any(|diagnostic| {
        diagnostic["code"] == "E1025"
            && diagnostic["path"] == "/nodes/publish/container/workdir"
            && diagnostic["message"] == "invalid path variable suffix: ../escape"
    }));
}
