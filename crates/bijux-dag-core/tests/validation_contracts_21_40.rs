use bijux_dag_core::{parse_graph_strict, Graph, Severity, ValidationDiagnostic};
use criterion as _;
use hex as _;
use serde as _;
use serde_json::json;
use serde_yaml as _;
use sha2 as _;
use std::collections::BTreeSet;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

fn fixture(name: &str) -> String {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name);
    std::fs::read_to_string(path).expect("read fixture")
}

fn diagnostics_json(diags: &[ValidationDiagnostic]) -> serde_json::Value {
    serde_json::to_value(diags).expect("serialize diagnostics")
}

#[test]
fn detects_cycle_dependencies() {
    let graph = parse_graph_strict(&fixture("cycle.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1004"));
}

#[test]
fn detects_unreachable_nodes() {
    let graph = parse_graph_strict(&fixture("unreachable.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "W2001" && d.severity == Severity::Warning));
}

#[test]
fn detects_duplicate_node_identifiers() {
    let graph = parse_graph_strict(&fixture("dup_id.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1001"));
}

#[test]
fn detects_conflicting_outputs_between_nodes() {
    let graph = parse_graph_strict(&fixture("output_collision.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1008"));
}

#[test]
fn detects_invalid_input_bindings_and_missing_references() {
    let graph = parse_graph_strict(&fixture("invalid_ref.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1020"));
}

#[test]
fn detects_non_existent_artifact_references() {
    let graph = parse_graph_strict(&fixture("unknown_node_output.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1021"));
}

#[test]
fn detects_duplicate_artifact_output_names() {
    let graph =
        parse_graph_strict(&fixture("duplicate_outputs_per_node.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1008"));
}

#[test]
fn detects_illegal_dependency_declarations() {
    let graph = parse_graph_strict(&fixture("forward_ref.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1022"));
}

#[test]
fn flags_invalid_root_topology_as_orphan_warning() {
    let graph = parse_graph_strict(&fixture("orphan.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "W2002"));
}

#[test]
fn covers_graph_with_no_executable_nodes_shape() {
    let raw = json!({
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"no-executable","owners":[],"tags":[]},
      "nodes":[],
      "edges":[]
    })
    .to_string();
    let graph = parse_graph_strict(&raw).expect("parse graph");
    assert!(
        graph.nodes.is_empty(),
        "empty node set must stay representable for explicit validation coverage"
    );
}

#[test]
fn validates_large_fan_out_graph() {
    let graph =
        parse_graph_strict(&fixture("graph_identity/large_fan_out.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(
        !diags.iter().any(|d| d.severity == Severity::Error),
        "fan-out fixture should remain valid"
    );
}

#[test]
fn validates_large_fan_in_graph() {
    let graph =
        parse_graph_strict(&fixture("graph_identity/large_fan_in.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    assert!(
        !diags.iter().any(|d| d.severity == Severity::Error),
        "fan-in fixture should remain valid"
    );
}

#[test]
fn validation_error_snapshot_shape_is_stable() {
    let graph = parse_graph_strict(&fixture("dup_id.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    let rendered = serde_json::to_string_pretty(&diagnostics_json(&diags)).expect("render");
    let expected = include_str!("snapshots/validation_error_snapshot.json").trim_end();
    assert_eq!(rendered, expected);
}

#[test]
fn validation_diagnostic_schema_is_stable() {
    let graph = parse_graph_strict(&fixture("invalid_ref.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    let payload = diagnostics_json(&diags);
    let first = payload.as_array().and_then(|items| items.first()).expect("diagnostic entry");

    let keys: BTreeSet<String> = first.as_object().expect("object").keys().cloned().collect();
    let expected: BTreeSet<String> =
        ["code", "message", "path", "hint", "severity"].iter().map(|s| s.to_string()).collect();
    assert_eq!(keys, expected);
}

#[test]
fn validation_stage_failure_classification_is_consistent() {
    let graph = parse_graph_strict(&fixture("invalid_ref.json")).expect("parse graph");
    let diags = graph.validate_with_warnings();
    for diag in diags {
        if diag.code.starts_with('E') {
            assert_eq!(diag.severity, Severity::Error);
        } else if diag.code.starts_with('W') {
            assert_eq!(diag.severity, Severity::Warning);
        }
    }
}

#[test]
fn validation_fuzz_invariants_hold_for_generated_graphs() {
    let mut seed: u64 = 0x5eed_cafe_d00d_beef;
    for _ in 0..200 {
        let nodes = 2 + (lcg(&mut seed) % 18) as usize;
        let graph = random_dag(nodes, &mut seed);
        let diags = graph.validate_with_warnings();

        let errs = diags.iter().filter(|d| d.severity == Severity::Error).count();
        assert!(errs == 0, "generated acyclic graph should be valid: {diags:?}");
    }
}

#[test]
fn validation_stress_thousands_of_nodes() {
    let graph = long_chain_graph(2_000);
    let diags = graph.validate_with_warnings();
    assert!(
        !diags.iter().any(|d| d.severity == Severity::Error),
        "large valid graph must remain valid"
    );
}

#[test]
fn validation_diagnostics_ordering_is_deterministic() {
    let graph = parse_graph_strict(&fixture("dup_id.json")).expect("parse graph");
    let first = graph.validate_with_warnings();
    let second = graph.validate_with_warnings();
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.code, b.code);
        assert_eq!(a.path, b.path);
        assert_eq!(a.message, b.message);
        assert_eq!(a.severity, b.severity);
    }
}

fn random_dag(nodes: usize, seed: &mut u64) -> Graph {
    let mut raw_nodes = Vec::with_capacity(nodes);
    let mut edges = Vec::new();

    for i in 0..nodes {
        raw_nodes.push(json!({
            "id": format!("n{i}"),
            "kind": "const",
            "inputs": if i == 0 { vec![] } else { vec!["in"] },
            "outputs": [{"name":"out","path":format!("n{i}/out.txt")}],
            "effects": ["filesystem"],
            "params": {"value": (lcg(seed) % 10_000) as i64}
        }));
    }

    for to in 1..nodes {
        let from = (lcg(seed) % to as u64) as usize;
        edges.push(json!({
            "from": {"node_id": format!("n{from}"), "port": "out"},
            "to": {"node_id": format!("n{to}"), "port": "in"}
        }));
    }

    let raw = json!({
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"fuzz","owners":[],"tags":[]},
      "nodes": raw_nodes,
      "edges": edges
    })
    .to_string();

    parse_graph_strict(&raw).expect("build random dag")
}

fn long_chain_graph(nodes: usize) -> Graph {
    let mut raw_nodes = Vec::with_capacity(nodes);
    let mut edges = Vec::with_capacity(nodes.saturating_sub(1));

    for i in 0..nodes {
        raw_nodes.push(json!({
            "id": format!("n{i}"),
            "kind": "const",
            "inputs": if i == 0 { vec![] } else { vec!["in"] },
            "outputs": [{"name":"out","path":format!("n{i}/out.txt")}],
            "effects": ["filesystem"],
            "params": {"value": i as i64}
        }));
        if i > 0 {
            edges.push(json!({
                "from": {"node_id": format!("n{}", i - 1), "port": "out"},
                "to": {"node_id": format!("n{i}"), "port": "in"}
            }));
        }
    }

    let raw = json!({
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"stress","owners":[],"tags":[]},
      "nodes": raw_nodes,
      "edges": edges
    })
    .to_string();

    parse_graph_strict(&raw).expect("build long chain")
}

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed
}
