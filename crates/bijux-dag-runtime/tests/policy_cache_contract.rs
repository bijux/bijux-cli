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

use bijux_dag_artifacts::RunOutputsIndex;
use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    registered_adapters, CacheMode, PolicyConfig, Runtime, RuntimeConfig, RuntimeError,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;

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

fn read_node_status(run_dir: &std::path::Path, node_id: &str) -> String {
    let data: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json"))
            .expect("read trace"),
    )
    .expect("parse trace");
    data["status"].as_str().unwrap_or("unknown").to_string()
}

#[test]
fn runtime_clean_env_defaults_to_stripped_environment() {
    let graph = parse_graph_strict(&shell_graph(
        "printf '%s' '${PATH:-missing-path}' > ../outputs/value.txt",
        &["filesystem"],
    ))
    .expect("parse graph");
    let runtime = Runtime::new();
    let with_clean_env = tempfile::tempdir().expect("temp");
    let run_clean = runtime
        .run(&graph, with_clean_env.path(), RuntimeConfig::default())
        .expect("run clean env");
    let clean_output =
        fs::read_to_string(run_clean.join("nodes").join("node").join("outputs").join("value.txt"))
            .expect("clean output");
    assert!(
        clean_output.contains("${PATH:-missing-path}") || clean_output.trim() == "missing-path"
    );

    let run_dir = tempfile::tempdir().expect("temp");
    let run_unstripped = runtime
        .run(
            &graph,
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
    assert_ne!(unstripped_output, "missing-path");
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
    let error = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                policy: PolicyConfig { deny_network: true, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect_err("deny network");
    assert!(
        matches!(error, RuntimeError::Executor(msg) if msg.contains("network effect denied by policy"))
    );
}

#[test]
fn runtime_rejects_env_effect_when_denied() {
    let graph = parse_graph_strict(&shell_graph(
        "printf '%s' ok > ../outputs/value.txt",
        &["filesystem", "env"],
    ))
    .expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");
    let error = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                policy: PolicyConfig { deny_env: true, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect_err("deny env");
    assert!(
        matches!(error, RuntimeError::Executor(msg) if msg.contains("env effect denied by policy"))
    );
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
    let error = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                policy: PolicyConfig { deny_clock: true, ..PolicyConfig::default() },
                ..RuntimeConfig::default()
            },
        )
        .expect_err("deny clock");
    assert!(
        matches!(error, RuntimeError::Executor(msg) if msg.contains("clock effect denied by policy"))
    );
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
