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

use bijux_dag_artifacts::{FailurePropagationRecord, RunOutputsIndex};
use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    registered_adapters, CacheMode, FailurePropagationMode, PolicyConfig, QueueIsolationPolicy,
    RunTimeoutBehavior, Runtime, RuntimeConfig, SchedulerFairness, SchedulerPolicy,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::sync::{Mutex, MutexGuard, OnceLock};

fn shell_graph(script: &str, effects: &[&str]) -> String {
    let effect_values: Vec<Value> =
        effects.iter().map(|effect| Value::String((*effect).to_string())).collect();

    let graph = json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "node",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "value.txt"}],
                "params": {
                    "argv": [
                        "/bin/sh",
                        "-c",
                        script,
                    ]
                },
                "effects": effect_values,
            }
        ],
        "edges": []
    });

    graph.to_string()
}

fn shell_graph_with_allowlist(script: &str, effects: &[&str], env_allowlist: &[&str]) -> String {
    let effect_values: Vec<Value> =
        effects.iter().map(|effect| Value::String((*effect).to_string())).collect();
    let env_values: Vec<Value> =
        env_allowlist.iter().map(|key| Value::String((*key).to_string())).collect();

    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "node",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "value.txt"}],
                "params": {
                    "argv": [
                        "/bin/sh",
                        "-c",
                        script
                    ]
                },
                "effects": effect_values,
                "env_allowlist": env_values,
            }
        ],
        "edges": []
    })
    .to_string()
}

fn process_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().expect("env lock")
}

fn graph_with_two_const_nodes() -> String {
    let graph = json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "a",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "a.txt"}],
                "params": {"value": "first"}
            },
            {
                "id": "b",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "b.txt"}],
                "params": {"value": "second"}
            }
        ],
        "edges": []
    });

    graph.to_string()
}

fn graph_with_failed_upstream_dependency() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "a",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "a.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "true"]},
                "effects": ["filesystem"]
            },
            {
                "id": "b",
                "kind": "const",
                "inputs": ["in"],
                "outputs": [{"name": "value", "path": "b.txt"}],
                "params": {"value": "downstream"}
            }
        ],
        "edges": [
            {"from": {"node_id": "a", "port": "value"}, "to": {"node_id": "b", "port": "in"}}
        ]
    })
    .to_string()
}

fn graph_with_fail_fast_abort() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "a",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "a.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "true"]},
                "effects": ["filesystem"]
            },
            {
                "id": "b",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "b.txt"}],
                "params": {"value": "never-run"}
            }
        ],
        "edges": []
    })
    .to_string()
}

fn graph_with_failure_propagation_merge() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "seed",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "seed.txt"}],
                "params": {"value": "seed"}
            },
            {
                "id": "fail",
                "kind": "shell",
                "inputs": ["in"],
                "outputs": [{"name": "value", "path": "fail.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "exit 1"]},
                "effects": ["filesystem"]
            },
            {
                "id": "ok",
                "kind": "shell",
                "inputs": ["in"],
                "outputs": [{"name": "value", "path": "ok.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "printf '%s' ok > ../outputs/ok.txt"]},
                "effects": ["filesystem"]
            },
            {
                "id": "join",
                "kind": "shell",
                "inputs": ["failed_branch", "healthy_branch"],
                "outputs": [{"name": "value", "path": "join.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "printf '%s' join > ../outputs/join.txt"]},
                "effects": ["filesystem"],
                "trigger_rule": "all_done"
            },
            {
                "id": "publish",
                "kind": "shell",
                "inputs": ["joined"],
                "outputs": [{"name": "value", "path": "publish.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "printf '%s' publish > ../outputs/publish.txt"]},
                "effects": ["filesystem"]
            },
            {
                "id": "independent",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "independent.txt"}],
                "params": {"value": "ready"}
            }
        ],
        "edges": [
            {"from": {"node_id": "seed", "port": "value"}, "to": {"node_id": "fail", "port": "in"}},
            {"from": {"node_id": "seed", "port": "value"}, "to": {"node_id": "ok", "port": "in"}},
            {"from": {"node_id": "fail", "port": "value"}, "to": {"node_id": "join", "port": "failed_branch"}},
            {"from": {"node_id": "ok", "port": "value"}, "to": {"node_id": "join", "port": "healthy_branch"}},
            {"from": {"node_id": "join", "port": "value"}, "to": {"node_id": "publish", "port": "joined"}}
        ]
    })
    .to_string()
}

fn graph_with_run_timeout_pending_node() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "a",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "a.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "sleep 0.5; printf '%s' ok > ../outputs/a.txt"]},
                "effects": ["filesystem"]
            },
            {
                "id": "b",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "b.txt"}],
                "params": {"value": "late"}
            }
        ],
        "edges": []
    })
    .to_string()
}

fn shell_graph_with_invalid_argv() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "node",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "value.txt"}],
                "params": {
                    "argv": "not-an-array"
                },
                "effects": ["filesystem"],
            }
        ],
        "edges": []
    })
    .to_string()
}

fn shell_retry_failure_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "node",
                "kind": "shell",
                "inputs": [],
                "outputs": [],
                "params": {
                    "argv": ["/bin/sh", "-c", "exit 1"]
                },
                "retry": {
                    "max_attempts": 2,
                    "backoff_ms": 0
                },
                "effects": ["filesystem"],
            }
        ],
        "edges": []
    })
    .to_string()
}

fn graph_with_cached_downstream_dependency(seed_value: &str) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "seed",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "seed.txt"}],
                "params": {"value": seed_value}
            },
            {
                "id": "consume",
                "kind": "shell",
                "inputs": ["in"],
                "outputs": [{"name": "value", "path": "result.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "cat ../inputs/seed/in > ../outputs/result.txt"]},
                "effects": ["filesystem"]
            }
        ],
        "edges": [
            {"from": {"node_id": "seed", "port": "value"}, "to": {"node_id": "consume", "port": "in"}}
        ]
    })
    .to_string()
}

fn read_node_status(run_dir: &std::path::Path, node_id: &str) -> String {
    let data: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json"))
            .expect("read trace"),
    )
    .expect("parse trace");
    data["status"].as_str().unwrap_or("unknown").to_string()
}

fn read_node_trace(run_dir: &std::path::Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json"))
            .expect("read trace"),
    )
    .expect("parse trace")
}

fn read_failure_propagation(run_dir: &std::path::Path) -> Vec<FailurePropagationRecord> {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("failure-propagation.json")).expect("failure propagation"),
    )
    .expect("parse failure propagation")
}

fn read_timeline(run_dir: &std::path::Path) -> Vec<Value> {
    serde_json::from_str::<Value>(
        &fs::read_to_string(run_dir.join("observability.timeline.json")).expect("timeline"),
    )
    .expect("parse timeline")["entries"]
        .as_array()
        .expect("timeline entries")
        .clone()
}

#[test]
fn runtime_clean_env_defaults_to_stripped_environment() {
    let _env_lock = process_env_lock();
    std::env::set_var("BIJUX_TEST_PATH", "declared-path");
    let undeclared_graph = parse_graph_strict(&shell_graph(
        "printf '%s' \"${BIJUX_TEST_PATH:-missing-path}\" > ../outputs/value.txt",
        &["filesystem"],
    ))
    .expect("parse graph");
    let runtime = Runtime::new();
    let with_clean_env = tempfile::tempdir().expect("temp");
    let run_clean = runtime
        .run(&undeclared_graph, with_clean_env.path(), RuntimeConfig::default())
        .expect("run clean env");
    let clean_output =
        fs::read_to_string(run_clean.join("nodes").join("node").join("outputs").join("value.txt"))
            .expect("clean output");
    assert_eq!(clean_output.trim(), "missing-path");

    let run_dir = tempfile::tempdir().expect("temp");
    let run_unstripped = runtime
        .run(
            &undeclared_graph,
            run_dir.path(),
            RuntimeConfig {
                policy: PolicyConfig { clean_env: false, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect("run unstripped env");
    let unstripped_output = fs::read_to_string(
        run_unstripped.join("nodes").join("node").join("outputs").join("value.txt"),
    )
    .expect("unstripped output");
    assert_eq!(unstripped_output.trim(), "missing-path");

    let declared_graph = parse_graph_strict(&shell_graph_with_allowlist(
        "printf '%s' \"$BIJUX_TEST_PATH\" > ../outputs/value.txt",
        &["filesystem", "env"],
        &["BIJUX_TEST_PATH"],
    ))
    .expect("parse declared graph");
    let declared_out = tempfile::tempdir().expect("declared out");
    let declared_run = runtime
        .run(
            &declared_graph,
            declared_out.path(),
            RuntimeConfig {
                policy: PolicyConfig { clean_env: false, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect("run declared env");
    let declared_output = fs::read_to_string(
        declared_run.join("nodes").join("node").join("outputs").join("value.txt"),
    )
    .expect("declared output");
    assert_eq!(declared_output.trim(), "declared-path");
    std::env::remove_var("BIJUX_TEST_PATH");
}

#[test]
fn runtime_requires_declared_exact_env_before_execution() {
    let _env_lock = process_env_lock();
    std::env::remove_var("BIJUX_REQUIRED_ENV");
    let graph = parse_graph_strict(&shell_graph_with_allowlist(
        "printf '%s' \"$BIJUX_REQUIRED_ENV\" > ../outputs/value.txt",
        &["filesystem", "env"],
        &["BIJUX_REQUIRED_ENV"],
    ))
    .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let error = runtime
        .run(&graph, out.path(), RuntimeConfig::default())
        .expect_err("missing env should fail before execution");
    let message = error.to_string();
    assert!(message.contains("missing required environment bindings"));
    assert!(message.contains("BIJUX_REQUIRED_ENV"));
    assert_eq!(fs::read_dir(out.path()).expect("out dir entries").count(), 0);
}

#[test]
fn runtime_rejects_network_effect_when_denied() {
    let graph = parse_graph_strict(&shell_graph(
        "printf '%s' ok > ../outputs/value.txt",
        &["filesystem", "network"],
    ))
    .expect("parse graph");
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
        .expect("deny network run");
    let trace = read_node_trace(&run_path, "node");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["kind"], "Policy");
    assert_eq!(trace["failure"]["details"]["effect"], "network");
    assert_eq!(trace["transition_cause"], "PolicyDenied");
    let propagation = read_failure_propagation(&run_path);
    assert_eq!(propagation[0].reason, "policy_denied");
}

#[test]
fn runtime_rejects_env_effect_when_denied() {
    let _env_lock = process_env_lock();
    std::env::set_var("BIJUX_DENIED_ENV", "present");
    let graph = parse_graph_strict(&shell_graph_with_allowlist(
        "printf '%s' ok > ../outputs/value.txt",
        &["filesystem", "env"],
        &["BIJUX_DENIED_ENV"],
    ))
    .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                policy: PolicyConfig { deny_env: true, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect("deny env run");
    let trace = read_node_trace(&run_path, "node");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["kind"], "Policy");
    assert_eq!(trace["failure"]["details"]["effect"], "env");
    assert_eq!(trace["transition_cause"], "PolicyDenied");
    std::env::remove_var("BIJUX_DENIED_ENV");
}

#[test]
fn runtime_rejects_clock_effect_when_denied() {
    let graph = parse_graph_strict(&shell_graph(
        "printf '%s' ok > ../outputs/value.txt",
        &["filesystem", "clock"],
    ))
    .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                policy: PolicyConfig { deny_clock: true, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect("deny clock run");
    let trace = read_node_trace(&run_path, "node");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["kind"], "Policy");
    assert_eq!(trace["failure"]["details"]["effect"], "clock");
    assert_eq!(trace["transition_cause"], "PolicyDenied");
}

#[test]
fn runtime_declared_outputs_index_includes_all_nodes() {
    let graph = parse_graph_strict(&graph_with_two_const_nodes()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");

    let outputs: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("outputs").join("index.json"))
            .expect("read outputs index"),
    )
    .expect("parse outputs index");
    let files = outputs["files"].as_array().expect("files");

    let actual: HashSet<String> =
        files.iter().map(|file| file["path"].as_str().unwrap_or("").to_string()).collect();
    let expected =
        HashSet::from(["nodes/a/outputs/a.txt".to_string(), "nodes/b/outputs/b.txt".to_string()]);
    assert_eq!(actual, expected);
    assert!(files.iter().all(|file| file["size_bytes"].as_u64().is_some_and(|size| size > 0)));
}

#[test]
fn runtime_missing_output_file_fails_with_missing_output() {
    let graph = parse_graph_strict(&shell_graph("true", &["filesystem"])).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    assert_eq!(manifest["status"], "failed");

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("nodes").join("node").join("trace.json")).expect("trace"),
    )
    .expect("parse trace");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "OUTPUT_MISSING");
    assert_eq!(trace["failure"]["class"], "user");
    assert_eq!(trace["transition_cause"], "MissingRequiredOutput");
    let propagation = read_failure_propagation(&run_path);
    assert_eq!(propagation[0].reason, "missing_required_output");
}

#[test]
fn runtime_cache_hit_uses_cached_nodes_when_mode_is_readwrite() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let cached_run = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("cache hit");
    let status = read_node_status(&cached_run, "node");
    assert_eq!(status, "cached");

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(cached_run.join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    let cached = manifest["node_counts"]["cached"].as_u64().unwrap_or(0);
    assert!(cached >= 1);
}

#[test]
fn runtime_cache_key_changes_when_materialized_input_hash_changes() {
    let first_graph =
        parse_graph_strict(&graph_with_cached_downstream_dependency("first")).expect("parse graph");
    let second_graph = parse_graph_strict(&graph_with_cached_downstream_dependency("second"))
        .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &first_graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");
    let initial_cache_entries = fs::read_dir(cache.path()).expect("cache entries").count();

    let rerun = runtime
        .run(
            &second_graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("rerun");

    assert_eq!(read_node_status(&rerun, "seed"), "success");
    assert_eq!(read_node_status(&rerun, "consume"), "success");
    let final_cache_entries = fs::read_dir(cache.path()).expect("cache entries").count();
    assert_eq!(initial_cache_entries, 2);
    assert_eq!(final_cache_entries, 4);
}

#[test]
fn runtime_cache_key_changes_when_policy_changes() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let run_with_different_policy = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                policy: PolicyConfig { deny_network: true, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect("policy-shift run");

    assert_eq!(read_node_status(&run_with_different_policy, "node"), "success");

    let cache_entries = fs::read_dir(cache.path()).expect("cache entries").count();
    assert_eq!(cache_entries, 2);
}

#[test]
fn runtime_cache_reuses_entries_across_operator_only_config_changes() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let cached_run = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                jobs: 4,
                cpu_budget: Some(8),
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                run_id: Some("operator-shaped-run".to_string()),
                submission_source: "automation".to_string(),
                trigger_source: "scheduler".to_string(),
                operator: "release-operator".to_string(),
                labels: vec!["nightly".to_string(), "priority:high".to_string()],
                scheduler_policy: SchedulerPolicy {
                    max_parallelism: 4,
                    cpu_budget: Some(8),
                    memory_budget_mb: None,
                    gpu_device_budget: None,
                    named_resource_capacities: std::collections::BTreeMap::new(),
                    fairness: SchedulerFairness::ThroughputPreferred,
                    queue_isolation: QueueIsolationPolicy::GroupIsolated,
                    bounded_executor_capacity: 8,
                    prefer_throughput_scheduler: true,
                },
                failure_propagation: FailurePropagationMode::ContinueIndependent,
                ..RuntimeConfig::default()
            },
        )
        .expect("cache hit");

    assert_eq!(read_node_status(&cached_run, "node"), "cached");
    let cache_entries = fs::read_dir(cache.path()).expect("cache entries").count();
    assert_eq!(cache_entries, 1);
}

#[test]
fn runtime_cache_key_changes_when_execution_contract_changes() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let rerun = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                node_timeout_ms: Some(1000),
                ..RuntimeConfig::default()
            },
        )
        .expect("execution contract rerun");

    assert_eq!(read_node_status(&rerun, "node"), "success");
    let cache_entries = fs::read_dir(cache.path()).expect("cache entries").count();
    assert_eq!(cache_entries, 2);
}

#[test]
fn runtime_cache_identity_tracks_declared_env_and_ignores_undeclared_env() {
    let _env_lock = process_env_lock();
    let graph = parse_graph_strict(&shell_graph_with_allowlist(
        "printf '%s' \"$BIJUX_DECLARED_CACHE_ENV\" > ../outputs/value.txt",
        &["filesystem", "env"],
        &["BIJUX_DECLARED_CACHE_ENV"],
    ))
    .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    std::env::set_var("BIJUX_DECLARED_CACHE_ENV", "alpha");
    std::env::set_var("BIJUX_UNDECLARED_CACHE_ENV", "noise-a");
    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    std::env::set_var("BIJUX_UNDECLARED_CACHE_ENV", "noise-b");
    let cached_run = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("cache hit");
    assert_eq!(read_node_status(&cached_run, "node"), "cached");
    let cached_output =
        fs::read_to_string(cached_run.join("nodes").join("node").join("outputs").join("value.txt"))
            .expect("cached output");
    assert_eq!(cached_output.trim(), "alpha");

    std::env::set_var("BIJUX_DECLARED_CACHE_ENV", "beta");
    let rerun = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("declared env change rerun");
    assert_eq!(read_node_status(&rerun, "node"), "success");
    let rerun_output =
        fs::read_to_string(rerun.join("nodes").join("node").join("outputs").join("value.txt"))
            .expect("rerun output");
    assert_eq!(rerun_output.trim(), "beta");

    std::env::remove_var("BIJUX_DECLARED_CACHE_ENV");
    std::env::remove_var("BIJUX_UNDECLARED_CACHE_ENV");
}

#[test]
fn runtime_artifacts_do_not_serialize_declared_env_values() {
    let _env_lock = process_env_lock();
    std::env::set_var("BIJUX_SECRET_VALUE", "top-secret-value");
    let graph = parse_graph_strict(&shell_graph_with_allowlist(
        "printf '%s' ok > ../outputs/value.txt",
        &["filesystem", "env"],
        &["BIJUX_SECRET_VALUE"],
    ))
    .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");

    for relative_path in ["manifest.json", "provenance.json", "nodes/node/trace.json"] {
        let payload =
            fs::read_to_string(run_path.join(relative_path)).expect("read runtime artifact");
        assert!(
            !payload.contains("top-secret-value"),
            "runtime artifact {relative_path} must not serialize declared env values"
        );
    }

    std::env::remove_var("BIJUX_SECRET_VALUE");
}

#[test]
fn runtime_cache_off_mode_never_uses_cached_nodes() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let run_off = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::Off,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("run off");
    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_off.join("nodes").join("node").join("trace.json")).expect("trace"),
    )
    .expect("parse trace");
    assert!(trace.get("cache_proof").is_none());

    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(run_off.join("manifest.json")).expect("manifest"))
            .expect("parse manifest");
    let cached = manifest["node_counts"]["cached"].as_u64().unwrap_or(0);
    assert_eq!(cached, 0);
}

#[test]
fn runtime_cache_verify_detects_corrupt_entry() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let cache_entry = fs::read_dir(cache.path())
        .expect("cache entries")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let outputs = path.join("outputs").join("value.txt");
            if outputs.exists() {
                Some(path)
            } else {
                None
            }
        })
        .next()
        .expect("cache value entry");

    let output = cache_entry.join("outputs").join("value.txt");
    fs::write(&output, b"corrupted").expect("corrupt cache");

    let run_recompute = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::Read,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("cache verify");

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_recompute.join("nodes").join("node").join("trace.json"))
            .expect("trace"),
    )
    .expect("parse trace");
    let proof = trace["cache_proof"].as_object().expect("cache proof");
    assert_eq!(proof["hit"], false);
    assert_eq!(proof["corrupt_detected"], true);
}

#[test]
fn runtime_cache_verify_rejects_missing_manifest_before_reuse() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let cache_entry = fs::read_dir(cache.path())
        .expect("cache entries")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| path.join("manifest.json").exists())
        .expect("cache manifest entry");
    fs::remove_file(cache_entry.join("manifest.json")).expect("remove cache manifest");

    let rerun = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::Read,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("cache verify");

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(rerun.join("nodes").join("node").join("trace.json")).expect("trace"),
    )
    .expect("parse trace");
    let proof = trace["cache_proof"].as_object().expect("cache proof");
    assert_eq!(proof["hit"], false);
    assert_eq!(proof["corrupt_detected"], true);
}

#[test]
fn runtime_cache_verify_rejects_incomplete_required_output_index() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let _ = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let cache_entry = fs::read_dir(cache.path())
        .expect("cache entries")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| path.join("outputs").join("index.json").exists())
        .expect("cache outputs entry");
    fs::write(cache_entry.join("outputs").join("index.json"), "{\"files\":[]}")
        .expect("truncate outputs index");

    let rerun = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::Read,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("cache verify");

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(rerun.join("nodes").join("node").join("trace.json")).expect("trace"),
    )
    .expect("parse trace");
    let proof = trace["cache_proof"].as_object().expect("cache proof");
    assert_eq!(proof["hit"], false);
    assert_eq!(proof["corrupt_detected"], true);
}

#[test]
fn runtime_cache_meta_records_strong_identity_components() {
    let graph =
        parse_graph_strict(&shell_graph("printf '%s' ok > ../outputs/value.txt", &["filesystem"]))
            .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let run_dir = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let meta_path = fs::read_dir(cache.path())
        .expect("cache entries")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let meta = path.join("meta.json");
            if meta.exists() {
                Some(meta)
            } else {
                None
            }
        })
        .next()
        .expect("cache meta");
    let manifest_path = meta_path.parent().expect("cache entry").join("manifest.json");
    let meta: Value =
        serde_json::from_str(&fs::read_to_string(meta_path).expect("cache meta json"))
            .expect("parse cache meta");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).expect("cache manifest json"))
            .expect("parse cache manifest");

    assert_eq!(meta["cache_metadata_version"], "cache-meta/v0.4");
    assert!(meta["cache_key"].is_string());
    assert!(meta["node_fingerprint"].is_string());
    assert!(meta["node_definition_fingerprint"].is_string());
    assert!(meta["declared_environment_fingerprint"].is_string());
    assert!(meta["input_lineage_fingerprint"].is_string());
    assert!(meta["params_fingerprint"].is_string());
    assert!(meta["command_fingerprint"].is_string());
    assert!(meta["policy_fingerprint"].is_string());
    assert!(meta["execution_contract_fingerprint"].is_string());
    assert!(meta["adapter_binary_sha256"].is_null());
    assert_eq!(meta["backend_class"], "local");
    assert_eq!(manifest["manifest_version"], "cache-entry/v0.1");
    assert_eq!(manifest["node_id"], "node");
    assert!(manifest["outputs"].as_array().is_some_and(|outputs| {
        outputs.iter().any(|output| output["path"] == "value.txt" && output["required"] == true)
    }));

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("node").join("trace.json")).expect("trace"),
    )
    .expect("parse trace");
    assert!(trace["cache_identity"]["cache_key"].is_string());
    assert!(trace["cache_identity"]["adapter_binary_sha256"].is_null());
    assert!(trace["cache_identity"]["params_fingerprint"].is_string());
    assert!(trace["cache_identity"]["command_fingerprint"].is_string());
}

#[test]
fn runtime_manifest_contains_expected_graph_fingerprint_and_counts() {
    let graph = parse_graph_strict(&graph_with_two_const_nodes()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    let expected_fp = graph.graph_fingerprint().expect("graph fingerprint");
    assert_eq!(manifest["graph_fingerprint"], expected_fp);
    assert!(manifest["planner_fingerprint"].is_string());
    assert!(manifest["execution_fingerprint"].is_string());
    assert!(manifest["evidence_fingerprint"].is_string());

    let counts = &manifest["node_counts"];
    let total = counts["success"].as_u64().unwrap_or(0)
        + counts["failed"].as_u64().unwrap_or(0)
        + counts["skipped"].as_u64().unwrap_or(0)
        + counts["cached"].as_u64().unwrap_or(0);
    assert_eq!(total, 2);

    let outputs: RunOutputsIndex = serde_json::from_str(
        &fs::read_to_string(run_path.join("outputs").join("index.json")).expect("run outputs"),
    )
    .expect("parse run outputs");
    assert_eq!(outputs.files.len(), 2);
}

#[test]
fn runtime_provenance_contains_identity_fingerprints() {
    let graph = parse_graph_strict(&graph_with_two_const_nodes()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");

    let provenance: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("provenance.json")).expect("provenance"),
    )
    .expect("parse provenance");
    assert_eq!(
        provenance["graph_fingerprint"],
        graph.graph_fingerprint().expect("graph fingerprint")
    );
    assert!(provenance["planner_fingerprint"].is_string());
    assert!(provenance["execution_fingerprint"].is_string());
    assert!(provenance["evidence_fingerprint"].is_string());
    assert!(provenance["runtime_fingerprint"].is_string());
    assert!(provenance["policy_fingerprint"].is_string());
}

#[test]
fn runtime_registered_adapters_expose_expected_builtins() {
    let adapters = registered_adapters();
    assert!(!adapters.is_empty());

    let ids: HashSet<String> = adapters.iter().map(|a| a.adapter_id.clone()).collect();

    assert!(ids.contains("const"));
    assert!(ids.contains("shell"));
    assert!(ids.contains("container"));

    let mut adapter_names = Vec::new();
    let mut seen = HashSet::new();
    for adapter in adapters {
        assert!(!adapter.adapter_id.is_empty());
        assert!(!adapter.adapter_version.is_empty());
        adapter_names.push(adapter.adapter_id);
    }
    for name in adapter_names {
        assert!(seen.insert(name));
    }
}

#[test]
fn runtime_failure_artifacts_include_traces_and_io_logs() {
    let graph = parse_graph_strict(&shell_graph("true", &["filesystem"])).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("failed run");

    assert!(run_path.join("nodes").join("node").join("stdout.log").exists());
    assert!(run_path.join("nodes").join("node").join("stderr.log").exists());
    assert!(run_path.join("nodes").join("node").join("trace.json").exists());
}

#[test]
fn runtime_adapter_errors_materialize_failure_logs() {
    let graph = parse_graph_strict(&shell_graph_with_invalid_argv()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("failed run");

    assert_eq!(read_node_status(&run_path, "node"), "failed");
    assert_eq!(
        fs::read_to_string(run_path.join("nodes").join("node").join("stdout.log")).expect("stdout"),
        ""
    );
    let stderr =
        fs::read_to_string(run_path.join("nodes").join("node").join("stderr.log")).expect("stderr");
    assert!(stderr.contains("argv must be an array of strings"));
    assert!(run_path.join("nodes").join("node").join("trace.json").exists());
}

#[test]
fn runtime_marks_upstream_blocked_nodes_as_dependency_failures() {
    let graph = parse_graph_strict(&graph_with_failed_upstream_dependency()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                failure_propagation: FailurePropagationMode::ContinueIndependent,
                ..RuntimeConfig::default()
            },
        )
        .expect("failed run");

    let downstream = read_node_trace(&run_path, "b");
    assert_eq!(downstream["status"], "failed");
    assert_eq!(downstream["failure"]["code"], "UPSTREAM_FAILED");
    assert_eq!(downstream["transition_cause"], "DependencyFailed");

    let propagation = read_failure_propagation(&run_path);
    assert!(propagation
        .iter()
        .any(|entry| entry.node_id == "b" && entry.reason == "upstream_failed"));
}

#[test]
fn runtime_fail_fast_marks_unscheduled_nodes_as_aborted_failures() {
    let graph = parse_graph_strict(&graph_with_fail_fast_abort()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                failure_propagation: FailurePropagationMode::FailFast,
                ..RuntimeConfig::default()
            },
        )
        .expect("failed run");

    let trace = read_node_trace(&run_path, "b");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "RUN_ABORTED");
    assert_eq!(trace["transition_cause"], "ExecutionAborted");
    assert_eq!(trace["lifecycle_state"], "cancelled");
    assert_eq!(
        trace["lifecycle_transitions"],
        serde_json::json!([
            {
                "from_state": "pending",
                "to_state": "ready",
                "cause": "scheduler_eligible",
                "unix_ms": trace["lifecycle_transitions"][0]["unix_ms"],
            },
            {
                "from_state": "ready",
                "to_state": "cancelled",
                "cause": "execution_aborted",
                "unix_ms": trace["lifecycle_transitions"][1]["unix_ms"],
            }
        ])
    );

    let propagation = read_failure_propagation(&run_path);
    assert!(propagation
        .iter()
        .any(|entry| entry.node_id == "b" && entry.reason == "execution_aborted"));

    let timeline = read_timeline(&run_path);
    assert!(timeline.iter().any(|entry| {
        entry["node_id"] == "b"
            && entry["label"] == "node_failed"
            && entry["reason"] == "execution_aborted"
    }));
}

#[test]
fn runtime_continue_independent_allows_all_done_join_after_failed_branch() {
    let graph = parse_graph_strict(&graph_with_failure_propagation_merge()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                failure_propagation: FailurePropagationMode::ContinueIndependent,
                ..RuntimeConfig::default()
            },
        )
        .expect("continue-independent run");

    assert_eq!(read_node_status(&run_path, "fail"), "failed");
    assert_eq!(read_node_status(&run_path, "ok"), "success");
    assert_eq!(read_node_status(&run_path, "join"), "success");
    assert_eq!(read_node_status(&run_path, "publish"), "success");
    assert_eq!(read_node_status(&run_path, "independent"), "success");

    let propagation = read_failure_propagation(&run_path);
    let failed_join_records =
        propagation.iter().filter(|entry| entry.node_id == "join" || entry.node_id == "publish");
    assert_eq!(failed_join_records.count(), 0);
}

#[test]
fn runtime_isolate_branch_skips_descendants_and_records_propagation_reason() {
    let graph = parse_graph_strict(&graph_with_failure_propagation_merge()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                failure_propagation: FailurePropagationMode::IsolateBranch,
                ..RuntimeConfig::default()
            },
        )
        .expect("isolate-branch run");

    assert_eq!(read_node_status(&run_path, "fail"), "failed");
    assert_eq!(read_node_status(&run_path, "ok"), "success");
    assert_eq!(read_node_status(&run_path, "join"), "skipped");
    assert_eq!(read_node_status(&run_path, "publish"), "skipped");
    assert_eq!(read_node_status(&run_path, "independent"), "success");

    let join = read_node_trace(&run_path, "join");
    assert_eq!(join["skip_reason"]["reason"], "isolated_branch_failure");
    assert_eq!(join["transition_cause"], "DependencyFailed");
    let publish = read_node_trace(&run_path, "publish");
    assert_eq!(publish["skip_reason"]["reason"], "isolated_branch_failure");
    assert_eq!(publish["transition_cause"], "DependencyFailed");

    let propagation = read_failure_propagation(&run_path);
    let join_record =
        propagation.iter().find(|entry| entry.node_id == "join").expect("join propagation");
    assert_eq!(join_record.status, "skipped");
    assert_eq!(join_record.reason, "isolated_branch_failure");
    assert_eq!(join_record.propagation_mode.as_deref(), Some("isolate_branch"));
    assert_eq!(join_record.blocking_nodes, vec!["fail".to_string()]);

    let publish_record =
        propagation.iter().find(|entry| entry.node_id == "publish").expect("publish propagation");
    assert_eq!(publish_record.status, "skipped");
    assert_eq!(publish_record.reason, "isolated_branch_failure");
    assert_eq!(publish_record.propagation_mode.as_deref(), Some("isolate_branch"));
    assert_eq!(publish_record.blocking_nodes, vec!["fail".to_string()]);
}

#[test]
fn runtime_replay_preserves_isolated_branch_propagation_decisions() {
    let graph = parse_graph_strict(&graph_with_failure_propagation_merge()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let original = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                failure_propagation: FailurePropagationMode::IsolateBranch,
                ..RuntimeConfig::default()
            },
        )
        .expect("original run");
    let original_manifest: Value = serde_json::from_str(
        &fs::read_to_string(original.join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    let parent_run_id = original_manifest["run_id"].as_str().expect("run id").to_string();

    let replay = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                parent_run_id: Some(parent_run_id.clone()),
                failure_propagation: FailurePropagationMode::IsolateBranch,
                ..RuntimeConfig::default()
            },
        )
        .expect("replay run");

    let replay_join = read_node_trace(&replay, "join");
    assert_eq!(replay_join["status"], "skipped");
    assert_eq!(replay_join["skip_reason"]["reason"], "isolated_branch_failure");
    assert_eq!(replay_join["replay_provenance"]["node_action"], "skipped");
    assert_eq!(replay_join["replay_provenance"]["source_run_id"], parent_run_id);

    let propagation = read_failure_propagation(&replay);
    let join_record =
        propagation.iter().find(|entry| entry.node_id == "join").expect("join propagation");
    assert_eq!(join_record.reason, "isolated_branch_failure");
    assert_eq!(join_record.propagation_mode.as_deref(), Some("isolate_branch"));
}

#[test]
fn runtime_marks_timed_out_runs_incomplete_when_deadline_is_exceeded() {
    let graph = parse_graph_strict(&graph_with_run_timeout_pending_node()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig { run_timeout_ms: Some(100), ..RuntimeConfig::default() },
        )
        .expect("timed run");

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    assert_eq!(manifest["status"], "timed_out");
    assert_eq!(manifest["run_timeout_behavior"], "finish_running");
    assert!(run_path.join(".run-incomplete.json").exists());
    assert!(!run_path.join(".run-complete.json").exists());

    let trace = read_node_trace(&run_path, "b");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "RUN_TIMEOUT");
    assert_eq!(trace["transition_cause"], "TimeoutExceeded");
    assert_eq!(trace["lifecycle_state"], "timed_out");
    assert_eq!(
        trace["lifecycle_transitions"],
        serde_json::json!([
            {
                "from_state": "pending",
                "to_state": "ready",
                "cause": "scheduler_eligible",
                "unix_ms": trace["lifecycle_transitions"][0]["unix_ms"],
            },
            {
                "from_state": "ready",
                "to_state": "timed_out",
                "cause": "timeout_exceeded",
                "unix_ms": trace["lifecycle_transitions"][1]["unix_ms"],
            }
        ])
    );

    let propagation = read_failure_propagation(&run_path);
    assert!(propagation
        .iter()
        .any(|entry| entry.node_id == "b" && entry.reason == "timeout_exceeded"));

    let timeline = read_timeline(&run_path);
    let timeout_idx = timeline
        .iter()
        .position(|entry| entry["label"] == "run_timed_out")
        .expect("run timeout timeline entry");
    let failed_idx = timeline
        .iter()
        .position(|entry| {
            entry["node_id"] == "b"
                && entry["label"] == "node_failed"
                && entry["reason"] == "timeout_exceeded"
        })
        .expect("timed out node timeline entry");
    assert!(timeout_idx < failed_idx);
}

#[test]
fn runtime_can_cancel_inflight_nodes_when_run_timeout_behavior_requires_it() {
    let graph = parse_graph_strict(&graph_with_run_timeout_pending_node()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                run_timeout_ms: Some(100),
                run_timeout_behavior: RunTimeoutBehavior::CancelRunning,
                ..RuntimeConfig::default()
            },
        )
        .expect("timed run");

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    assert_eq!(manifest["status"], "timed_out");
    assert_eq!(manifest["run_timeout_behavior"], "cancel_running");
    assert!(run_path.join(".run-incomplete.json").exists());
    assert!(!run_path.join(".run-complete.json").exists());

    let running_trace = read_node_trace(&run_path, "a");
    assert_eq!(running_trace["status"], "failed");
    assert_eq!(running_trace["failure"]["code"], "EXEC_TIMEOUT");
    assert_eq!(running_trace["failure"]["class"], "timeout");
    assert_eq!(running_trace["lifecycle_state"], "timed_out");

    let pending_trace = read_node_trace(&run_path, "b");
    assert_eq!(pending_trace["failure"]["code"], "RUN_TIMEOUT");
    assert_eq!(pending_trace["failure"]["class"], "timeout");
    assert_eq!(pending_trace["lifecycle_state"], "timed_out");
}

#[test]
fn runtime_persists_node_attempt_history() {
    let graph = parse_graph_strict(&shell_retry_failure_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("failed run");

    let attempts: Value = serde_json::from_str(
        &fs::read_to_string(run_path.join("nodes").join("node").join("attempts.json"))
            .expect("attempts"),
    )
    .expect("parse attempts");
    let entries = attempts.as_array().expect("array");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["attempt"], 1);
    assert_eq!(entries[2]["attempt"], 3);
    assert!(entries.iter().all(|entry| entry["status"] == "Failed"));
}
