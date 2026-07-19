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
use bijux_dag_runtime::{CacheMode, Runtime, RuntimeConfig};
use serde_json::Value;
use std::fs;

fn graph_text() -> String {
    r#"{
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {
          "id": "const1",
          "kind": "const",
          "inputs": [],
          "outputs": [{"name": "value", "path": "value.txt"}],
          "params": {"value": "hello"}
        }
      ],
      "edges": []
    }"#
    .to_string()
}

fn stable_manifest_shape(path: &std::path::Path) -> Value {
    let mut value: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    value["run_id"] = Value::String("<normalized>".to_string());
    value["created_unix_ms"] = Value::Number(0u64.into());
    value["started_unix_ms"] = Value::Number(0u64.into());
    value["finished_unix_ms"] = Value::Number(0u64.into());
    value
}

#[test]
fn manifest_shape_stable_across_replay_like_reexecution() {
    let graph = parse_graph_strict(&graph_text()).unwrap();
    let runtime = Runtime::new();
    let out = tempfile::tempdir().unwrap();

    let run_a = runtime.run(&graph, out.path(), RuntimeConfig::default()).unwrap();
    let run_b = runtime.run(&graph, out.path(), RuntimeConfig::default()).unwrap();

    let a = stable_manifest_shape(&run_a.join("manifest.json"));
    let b = stable_manifest_shape(&run_b.join("manifest.json"));
    assert_eq!(a, b);
}

#[test]
fn manifest_shape_stable_between_uncached_and_cached_runs() {
    let graph = parse_graph_strict(&graph_text()).unwrap();
    let runtime = Runtime::new();
    let out = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();

    let uncached = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::Off,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .unwrap();

    let cached = runtime
        .run(
            &graph,
            out.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .unwrap();

    let uncached_manifest = stable_manifest_shape(&uncached.join("manifest.json"));
    let cached_manifest = stable_manifest_shape(&cached.join("manifest.json"));

    assert_eq!(uncached_manifest["graph_fingerprint"], cached_manifest["graph_fingerprint"]);
    assert_eq!(uncached_manifest["spec"], cached_manifest["spec"]);
    assert!(cached_manifest["node_counts"]["cached"].as_u64().is_some());
}
