use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{parse_graph_strict, Severity};
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    fs::read_to_string(path).unwrap()
}

fn has_code(diags: &[bijux_dag_core::ValidationDiagnostic], code: &str) -> bool {
    diags.iter().any(|d| d.code == code)
}

#[test]
fn fixture_dup_id() {
    let input = fixture("dup_id.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1001"));
}

#[test]
fn fixture_dangling() {
    let input = fixture("dangling.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1002"));
}

#[test]
fn fixture_cycle() {
    let input = fixture("cycle.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1004"));
}

#[test]
fn fixture_illegal_id() {
    let input = fixture("illegal_id.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1007"));
}

#[test]
fn fixture_unreachable() {
    let input = fixture("unreachable.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "W2001" && d.severity == Severity::Warning));
}

#[test]
fn fixture_orphan() {
    let input = fixture("orphan.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "W2002" && d.severity == Severity::Warning));
}

#[test]
fn fixture_output_collision() {
    let input = fixture("output_collision.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1008"));
}

#[test]
fn fixture_missing_effects() {
    let input = fixture("missing_effects.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1009"));
}

#[test]
fn fixture_env_allowlist_no_env() {
    let input = fixture("env_allowlist_no_env.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1010"));
}

#[test]
fn fixture_shell_no_filesystem() {
    let input = fixture("shell_no_fs.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1009"));
}

#[test]
fn fixture_invalid_ref() {
    let input = fixture("invalid_ref.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1020"));
}

#[test]
fn fixture_retry_nondet() {
    let input = fixture("retry_nondet.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1011"));
}

#[test]
fn fixture_conflicting_retry_timeout_policy() {
    let input = fixture("conflicting_retry_timeout_policy.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(
        has_code(&diags, "E1011"),
        "non-deterministic shell node with retries must be rejected"
    );
}

#[test]
fn fixture_unknown_node_output() {
    let input = fixture("unknown_node_output.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1021"));
}

#[test]
fn fixture_forward_ref() {
    let input = fixture("forward_ref.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1022"));
}

#[test]
fn fixture_invalid_env_reference() {
    let input = fixture("invalid_env_reference.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1010"));
}

#[test]
fn env_effect_requires_declared_allowlist_bindings() {
    let input = r#"{
      "spec":"bijux-dag/v0.1",
      "nodes":[
        {
          "id":"env-node",
          "kind":"shell",
          "inputs":[],
          "outputs":[{"name":"out","path":"out.txt"}],
          "params":{"argv":["/bin/sh","-c","echo ok > ../outputs/out.txt"]},
          "effects":["filesystem","env"]
        }
      ],
      "edges":[]
    }"#;
    let graph = parse_graph_strict(input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1035"));
}

#[test]
fn fixture_invalid_resource_declaration_rejects_unknown_resource_keys() {
    let input = fixture("invalid_resource_declaration.json");
    let error = parse_graph_strict(&input).expect_err("unknown resources key must be rejected");
    let message = error.to_string();
    assert!(
        message.contains("resources") || message.contains("gpu"),
        "error should describe invalid resource declaration, got: {message}"
    );
}

#[test]
fn fixture_duplicate_outputs_per_node() {
    let input = fixture("duplicate_outputs_per_node.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1008"));
}

#[test]
fn fixture_missing_required_input_binding() {
    let input = fixture("missing_required_input_binding.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1005"));
}

#[test]
fn fixture_illegal_output_path_traversal() {
    let input = fixture("illegal_output_path_traversal.json");
    let error = parse_graph_strict(&input).expect_err("path traversal must be rejected");
    assert_eq!(error.to_string(), "validation failed");
}

#[test]
fn fixture_invalid_container_workdir() {
    let input = fixture("invalid_container_workdir.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1025"));
}

#[test]
fn fixture_unsupported_node_settings() {
    let input = fixture("unsupported_node_settings.json");
    let graph = parse_graph_strict(&input).unwrap();
    let diags = graph.validate_with_warnings();
    assert!(has_code(&diags, "E1024"));
}
