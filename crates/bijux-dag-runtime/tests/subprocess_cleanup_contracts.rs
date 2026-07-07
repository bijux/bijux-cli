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

use bijux_dag_artifacts::{write_json_atomic_durable, RunStopRequest};
use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{Runtime, RuntimeConfig};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

fn cleanup_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn cleanup_command(marker_path: &Path, require_prepare_input: bool) -> Vec<String> {
    let mut command = String::new();
    if require_prepare_input {
        command.push_str("cat ../inputs/prepare/in >/dev/null; ");
    }
    command.push_str("( /bin/sh -c 'sleep 1; printf orphan > \"$1\"' sh \"$1\" & wait ) & sleep 5");
    vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        command,
        "sh".to_string(),
        marker_path.display().to_string(),
    ]
}

fn timeout_cleanup_graph(marker_path: &Path) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "execute",
                "kind": "shell",
                "outputs": [],
                "params": {
                    "argv": cleanup_command(marker_path, false),
                    "timeout_ms": 100
                },
                "effects": ["filesystem"]
            }
        ],
        "edges": []
    })
    .to_string()
}

fn cancellation_cleanup_graph(marker_path: &Path) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "prepare",
                "kind": "const",
                "outputs": [{"name": "value", "path": "prepare.txt"}],
                "params": {"value": "ready"}
            },
            {
                "id": "execute",
                "kind": "shell",
                "inputs": ["in"],
                "outputs": [],
                "params": {
                    "argv": cleanup_command(marker_path, true)
                },
                "effects": ["filesystem"]
            }
        ],
        "edges": [
            {"from": {"node_id": "prepare", "port": "value"}, "to": {"node_id": "execute", "port": "in"}}
        ]
    })
    .to_string()
}

fn read_json(path: PathBuf) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse json")
}

fn wait_for_path(path: &Path) {
    for _ in 0..200 {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("path did not appear in time: {}", path.display());
}

#[test]
fn timed_out_shell_node_does_not_leave_background_descendants() {
    let _guard = cleanup_test_lock().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::tempdir().expect("temp dir");
    let marker_path = root.path().join("timeout-orphan.txt");
    let graph = parse_graph_strict(&timeout_cleanup_graph(&marker_path)).expect("parse graph");

    let run_path =
        Runtime::new().run(&graph, root.path(), RuntimeConfig::default()).expect("timed out run");

    let manifest = read_json(run_path.join("manifest.json"));
    assert_eq!(manifest["status"], "failed");

    let execute = read_json(run_path.join("nodes").join("execute").join("trace.json"));
    assert_eq!(execute["status"], "failed");
    assert_eq!(execute["failure"]["code"], "EXEC_TIMEOUT");
    assert_eq!(execute["lifecycle_state"], "timed_out");

    thread::sleep(Duration::from_millis(1_500));
    assert!(!marker_path.exists(), "timed out shell node left a background descendant running");
}

#[test]
fn stop_request_cancellation_does_not_leave_background_descendants() {
    let _guard = cleanup_test_lock().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::tempdir().expect("temp dir");
    let marker_path = root.path().join("cancel-orphan.txt");
    let graph_json = cancellation_cleanup_graph(&marker_path);
    let run_root = root.path().to_path_buf();
    let run_id = "cleanup-stop-request";
    let staging_dir = root.path().join("run.tmp-cleanup-stop-request");
    let stop_request_path = staging_dir.join("run.stop-request.json");

    let runner = thread::spawn(move || {
        let graph = parse_graph_strict(&graph_json).expect("parse graph");
        Runtime::new().run(
            &graph,
            &run_root,
            RuntimeConfig { jobs: 1, run_id: Some(run_id.to_string()), ..RuntimeConfig::default() },
        )
    });

    wait_for_path(&staging_dir.join("nodes").join("execute").join("work"));

    let request = RunStopRequest {
        schema_version: "run-stop-request/v0.1".to_string(),
        run_id: run_id.to_string(),
        requested_unix_ms: 42,
        source: "cli".to_string(),
        reason: None,
    };
    write_json_atomic_durable(
        &stop_request_path,
        &serde_json::to_value(&request).expect("request json"),
    )
    .expect("write stop request");

    let run_path = runner.join().expect("runner thread").expect("cancelled run");
    let manifest = read_json(run_path.join("manifest.json"));
    assert_eq!(manifest["status"], "cancelled");
    assert_eq!(manifest["run_cancellation_cause"], "operator_request");

    let execute = read_json(run_path.join("nodes").join("execute").join("trace.json"));
    assert_eq!(execute["status"], "cancelled");
    assert_eq!(execute["failure"]["code"], "EXEC_CANCELLED");
    assert_eq!(execute["lifecycle_state"], "cancelled");

    thread::sleep(Duration::from_millis(1_500));
    assert!(!marker_path.exists(), "cancelled shell node left a background descendant running");
}
