use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{PolicyConfig, Runtime, RuntimeConfig};
use serde_json::{json, Value};
use std::fs;

fn retry_success_graph(backoff_ms: u64) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nondeterminism_allowed": true,
        "nodes": [
            {
                "id": "worker",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "worker.txt"}],
                "retry": {"max_attempts": 1, "backoff_ms": backoff_ms},
                "effects": ["filesystem"],
                "params": {
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "if [ ! -f marker ]; then touch marker; echo first-attempt >&2; exit 1; fi; echo recovered; printf '%s' ok > ../outputs/worker.txt"
                    ]
                }
            }
        ],
        "edges": []
    })
    .to_string()
}

fn retry_failure_graph(backoff_ms: u64) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "worker",
                "kind": "shell",
                "inputs": [],
                "outputs": [],
                "retry": {"max_attempts": 1, "backoff_ms": backoff_ms},
                "effects": ["filesystem"],
                "params": {
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "echo always-fail >&2; exit 1"
                    ]
                }
            }
        ],
        "edges": []
    })
    .to_string()
}

fn retry_ineligible_user_failure_graph(backoff_ms: u64) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "worker",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "worker.txt"}],
                "retry": {"max_attempts": 2, "backoff_ms": backoff_ms},
                "effects": ["filesystem"],
                "params": {
                    "retryable_failure_classes": ["execution"],
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "printf 'completed without declared output'"
                    ]
                }
            }
        ],
        "edges": []
    })
    .to_string()
}

fn retryable_exit_code_graph(backoff_ms: u64) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "worker",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "worker.txt"}],
                "retry": {"max_attempts": 1, "backoff_ms": backoff_ms},
                "effects": ["filesystem"],
                "params": {
                    "retryable_failure_classes": ["timeout"],
                    "retryable_exit_codes": [75],
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "if [ ! -f marker ]; then touch marker; echo temporary >&2; exit 75; fi; printf '%s' ok > ../outputs/worker.txt"
                    ]
                }
            }
        ],
        "edges": []
    })
    .to_string()
}

fn timeout_retry_graph(timeout_retry_policy: &str) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "worker",
                "kind": "shell",
                "inputs": [],
                "outputs": [],
                "retry": {"max_attempts": 1, "backoff_ms": 10},
                "effects": ["filesystem"],
                "timeout_ms": 5,
                "params": {
                    "timeout_retry_policy": timeout_retry_policy,
                    "retryable_failure_classes": [],
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "sleep 1"
                    ]
                }
            }
        ],
        "edges": []
    })
    .to_string()
}

fn policy_denial_retry_graph(backoff_ms: u64) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nondeterminism_allowed": true,
        "nodes": [
            {
                "id": "worker",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "worker.txt"}],
                "retry": {"max_attempts": 2, "backoff_ms": backoff_ms},
                "effects": ["filesystem", "network"],
                "params": {
                    "retryable_failure_classes": ["policy"],
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "printf '%s' ok > ../outputs/worker.txt"
                    ]
                }
            }
        ],
        "edges": []
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

fn read_attempts(run_dir: &std::path::Path, node_id: &str) -> Vec<Value> {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("attempts.json"))
            .expect("read attempts"),
    )
    .expect("parse attempts")
}

#[test]
fn retry_persists_separate_attempt_logs_and_backoff_evidence() {
    let graph = parse_graph_strict(&retry_success_graph(40)).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("retry run");

    let trace = read_node_trace(&run_path, "worker");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["attempt"], 2);

    let attempts = read_attempts(&run_path, "worker");
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0]["attempt"], 1);
    assert_eq!(attempts[0]["scheduled_backoff_ms"], 40);
    assert_eq!(attempts[0]["stderr_path"], "attempts/1/stderr.log");
    assert_eq!(attempts[1]["attempt"], 2);
    assert!(attempts[1].get("scheduled_backoff_ms").is_none());
    assert_eq!(attempts[1]["stdout_path"], "attempts/2/stdout.log");

    let first_finished = attempts[0]["finished_unix_ms"].as_u64().expect("first finished");
    let second_started = attempts[1]["started_unix_ms"].as_u64().expect("second started");
    assert!(
        second_started >= first_finished.saturating_add(35),
        "retry backoff was not honored: first_finished={first_finished}, second_started={second_started}"
    );

    let first_stderr = fs::read_to_string(
        run_path.join("nodes").join("worker").join("attempts").join("1").join("stderr.log"),
    )
    .expect("first stderr");
    assert!(first_stderr.contains("first-attempt"));

    let second_stdout = fs::read_to_string(
        run_path.join("nodes").join("worker").join("attempts").join("2").join("stdout.log"),
    )
    .expect("second stdout");
    assert!(second_stdout.contains("recovered"));

    let run_log = fs::read_to_string(run_path.join("run.log.jsonl")).expect("run log");
    assert!(run_log.contains("\"event\":\"node_retry_scheduled\""));

    let timeline: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("observability.timeline.json")).expect("timeline"),
    )
    .expect("parse timeline");
    assert!(timeline["entries"]
        .as_array()
        .expect("timeline entries")
        .iter()
        .any(|entry| entry["label"] == "node_retry_scheduled" && entry["node_id"] == "worker"));
}

#[test]
fn retry_exhaustion_records_final_attempt_and_retry_exhausted_event() {
    let graph = parse_graph_strict(&retry_failure_graph(25)).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path =
        runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("failed retry run");

    let trace = read_node_trace(&run_path, "worker");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["attempt"], 2);

    let attempts = read_attempts(&run_path, "worker");
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0]["scheduled_backoff_ms"], 25);
    assert!(attempts[1]["failure"].is_object());

    let run_log = fs::read_to_string(run_path.join("run.log.jsonl")).expect("run log");
    assert!(run_log.contains("\"event\":\"node_retry_exhausted\""));
}

#[test]
fn retry_stops_when_failure_class_is_not_retry_eligible() {
    let graph = parse_graph_strict(&retry_ineligible_user_failure_graph(25)).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("failed run");

    let trace = read_node_trace(&run_path, "worker");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["attempt"], 1);
    assert_eq!(trace["failure"]["code"], "OUTPUT_MISSING");
    assert_eq!(trace["failure"]["class"], "user");

    let attempts = read_attempts(&run_path, "worker");
    assert_eq!(attempts.len(), 1);
    assert!(attempts[0].get("scheduled_backoff_ms").is_none());
    assert_eq!(attempts[0]["failure"]["class"], "user");

    let run_log = fs::read_to_string(run_path.join("run.log.jsonl")).expect("run log");
    assert!(!run_log.contains("\"event\":\"node_retry_scheduled\""));
    assert!(run_log.contains("\"event\":\"node_retry_exhausted\""));
    assert!(run_log.contains("\"failure_code\":\"OUTPUT_MISSING\""));
}

#[test]
fn retry_can_be_enabled_by_explicit_exit_code_rule() {
    let graph = parse_graph_strict(&retryable_exit_code_graph(15)).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("retry run");

    let trace = read_node_trace(&run_path, "worker");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["attempt"], 2);

    let attempts = read_attempts(&run_path, "worker");
    assert_eq!(attempts[0]["retry_decision"]["reason"], "retryable_exit_code_matched");
    assert_eq!(attempts[0]["retry_decision"]["matched_exit_code"], 75);
    assert_eq!(attempts[0]["scheduled_backoff_ms"], 15);

    let run_log = fs::read_to_string(run_path.join("run.log.jsonl")).expect("run log");
    assert!(run_log.contains("\"retry_reason\":\"retryable_exit_code_matched\""));
    assert!(run_log.contains("\"matched_exit_code\":75"));
}

#[test]
fn timeout_retry_policy_can_disable_timeout_retries() {
    let graph = parse_graph_strict(&timeout_retry_graph("never")).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");

    let trace = read_node_trace(&run_path, "worker");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["attempt"], 1);
    assert_eq!(trace["failure"]["class"], "timeout");

    let attempts = read_attempts(&run_path, "worker");
    assert_eq!(
        attempts[0]["retry_decision"]["reason"],
        "timeout_retry_policy_denies_timeout_retry"
    );
    assert!(attempts[0].get("scheduled_backoff_ms").is_none());
}

#[test]
fn timeout_retry_policy_can_force_timeout_retries() {
    let graph = parse_graph_strict(&timeout_retry_graph("always")).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");

    let trace = read_node_trace(&run_path, "worker");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["attempt"], 2);

    let attempts = read_attempts(&run_path, "worker");
    assert_eq!(
        attempts[0]["retry_decision"]["reason"],
        "timeout_retry_policy_allows_timeout_retry"
    );
    assert_eq!(attempts[0]["scheduled_backoff_ms"], 10);
}

#[test]
fn policy_failures_never_retry_even_when_policy_class_is_declared_retryable() {
    let graph = parse_graph_strict(&policy_denial_retry_graph(20)).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                policy: PolicyConfig { deny_network: true, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect("run");

    let trace = read_node_trace(&run_path, "worker");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["attempt"], 1);
    assert_eq!(trace["failure"]["class"], "policy");

    let attempts = read_attempts(&run_path, "worker");
    assert_eq!(attempts[0]["retry_decision"]["reason"], "policy_failures_are_non_retryable");
    assert!(attempts[0].get("scheduled_backoff_ms").is_none());

    let run_log = fs::read_to_string(run_path.join("run.log.jsonl")).expect("run log");
    assert!(!run_log.contains("\"event\":\"node_retry_scheduled\""));
    assert!(run_log.contains("\"retry_reason\":\"policy_failures_are_non_retryable\""));
}
