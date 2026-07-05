use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::cancellation_is_terminal;
use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{Runtime, RuntimeConfig};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

fn cancellation_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn operator_cancellation_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "prepare",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "prepare.txt"}],
                "params": {"value": "ready"}
            },
            {
                "id": "execute",
                "kind": "shell",
                "inputs": ["in"],
                "outputs": [{"name": "value", "path": "execute.txt"}],
                "params": {
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "sleep 2; cat ../inputs/prepare/in > ../outputs/execute.txt"
                    ]
                },
                "effects": ["filesystem"]
            },
            {
                "id": "publish",
                "kind": "shell",
                "inputs": ["in"],
                "outputs": [{"name": "value", "path": "publish.txt"}],
                "params": {
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "cat ../inputs/execute/in > ../outputs/publish.txt"
                    ]
                },
                "effects": ["filesystem"]
            }
        ],
        "edges": [
            {"from": {"node_id": "prepare", "port": "value"}, "to": {"node_id": "execute", "port": "in"}},
            {"from": {"node_id": "execute", "port": "value"}, "to": {"node_id": "publish", "port": "in"}}
        ]
    })
    .to_string()
}

fn read_node_trace(run_dir: &std::path::Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json"))
            .expect("read trace"),
    )
    .expect("parse trace")
}

#[test]
fn cancellation_requires_terminal_node_state() {
    assert!(cancellation_is_terminal(true, true));
    assert!(!cancellation_is_terminal(true, false));
}

#[test]
fn operator_cancellation_preserves_completed_nodes_and_marks_remaining_nodes_cancelled() {
    let _guard = cancellation_test_lock().lock().expect("cancellation test lock");
    let graph = parse_graph_strict(&operator_cancellation_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");

    let signaler = thread::spawn(|| {
        thread::sleep(Duration::from_millis(150));
        let status = Command::new("kill")
            .args(["-INT", &std::process::id().to_string()])
            .status()
            .expect("send interrupt");
        assert!(status.success(), "interrupt delivery failed");
    });

    let run_path = runtime
        .run(&graph, out.path(), RuntimeConfig { jobs: 1, ..RuntimeConfig::default() })
        .expect("cancelled run");
    signaler.join().expect("signaler thread");

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    assert_eq!(manifest["status"], "cancelled");
    assert_eq!(manifest["run_cancellation_cause"], "operator_interrupt");
    assert_eq!(manifest["node_counts"]["success"], 1);
    assert_eq!(manifest["node_counts"]["cancelled"], 2);

    let prepare = read_node_trace(&run_path, "prepare");
    assert_eq!(prepare["status"], "success");

    let execute = read_node_trace(&run_path, "execute");
    assert_eq!(execute["status"], "cancelled");
    assert_eq!(execute["failure"]["code"], "EXEC_CANCELLED");
    assert_eq!(execute["lifecycle_state"], "cancelled");
    assert_eq!(execute["transition_cause"], "CancelRequested");

    let publish = read_node_trace(&run_path, "publish");
    assert_eq!(publish["status"], "cancelled");
    assert_eq!(publish["skip_reason"]["reason"], "cancelled");
    assert_eq!(publish["lifecycle_state"], "cancelled");
    assert_eq!(publish["transition_cause"], "CancelRequested");
}
