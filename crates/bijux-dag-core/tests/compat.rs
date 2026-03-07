use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{parse_graph_strict, Graph};
use std::fs;
use std::path::PathBuf;

fn compat_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("compat");
    path.push("v0.1");
    path.push(name);
    path
}

fn load_graph() -> Graph {
    let data = fs::read_to_string(compat_path("simple.dag.json")).unwrap();
    parse_graph_strict(&data).unwrap()
}

#[test]
fn compat_canonical_matches() {
    let graph = load_graph();
    let expected = fs::read_to_string(compat_path("simple.canonical.json")).unwrap();
    let actual = graph.to_canonical_json().unwrap();
    assert_eq!(expected.trim(), actual.trim());
}

#[test]
fn compat_fingerprints_match() {
    let graph = load_graph();
    let expected_fp = fs::read_to_string(compat_path("simple.graph_fingerprint")).unwrap();
    let fp = graph.graph_fingerprint().unwrap();
    assert_eq!(expected_fp.trim(), fp);

    let expected_nodes: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(compat_path("simple.node_fingerprints.json")).unwrap(),
    )
    .unwrap();
    let resolved = graph.resolve_graph().unwrap().resolved_params;
    let mut nodes = serde_json::Map::new();
    for n in &graph.nodes {
        let params = resolved.get(&n.id).unwrap();
        let fp = graph.node_fingerprint_with_params(n, params).unwrap();
        nodes.insert(n.id.clone(), serde_json::Value::String(fp));
    }
    let actual = serde_json::Value::Object(nodes);
    assert_eq!(expected_nodes, actual);
}
