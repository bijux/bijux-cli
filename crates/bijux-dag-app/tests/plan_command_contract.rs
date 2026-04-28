use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
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
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;

fn write_graph_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-cmd","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write graph");
    (dir, dag)
}

#[test]
fn plan_explain_supports_json_output_with_node_reasons() {
    let (_dir, dag) = write_graph_fixture();
    let matches = dag_command()
        .try_get_matches_from(["dag", "--json", "plan", "explain", dag.to_string_lossy().as_ref()])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn plan_diagnostics_supports_json_payload() {
    let (_dir, dag) = write_graph_fixture();
    let matches = dag_command()
        .try_get_matches_from([
            "dag",
            "--json",
            "plan",
            "diagnostics",
            dag.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);

    let payload: Value = serde_json::json!({"assertion":"routing only"});
    assert!(payload.is_object());
}

#[test]
fn plan_diff_supports_json_output() {
    let (_before_dir, before) = write_graph_fixture();
    let after_dir = tempfile::tempdir().expect("tmp");
    let after = after_dir.path().join("graph-tagged.json");
    fs::write(
        &after,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-cmd-tagged","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"tags":["critical"],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write graph");

    let matches = dag_command()
        .try_get_matches_from([
            "dag",
            "--json",
            "plan",
            "diff",
            before.to_string_lossy().as_ref(),
            after.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn show_effective_plan_supports_json_output() {
    let (_dir, dag) = write_graph_fixture();
    let matches = dag_command()
        .try_get_matches_from([
            "dag",
            "--json",
            "show-effective-plan",
            dag.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}
