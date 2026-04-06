use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_core::GraphError;
use std::path::{Path, PathBuf};

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/graph_identity")
        .canonicalize()
        .expect("graph identity fixtures root")
}

fn parse_fixture(path: &Path) -> bijux_dag_core::Graph {
    let payload = std::fs::read_to_string(path).expect("read fixture");
    parse_graph_strict(&payload).expect("parse fixture")
}

#[test]
fn canonical_graph_bytes_are_deterministic_property_contract() {
    let path = fixtures_root().join("deep_dependency_tree.json");
    let graph = parse_fixture(&path);
    let first = graph.canonical_json_bytes().expect("canonical bytes 1");
    for _ in 0..25 {
        let next = graph.canonical_json_bytes().expect("canonical bytes n");
        assert_eq!(first, next);
    }
}

#[test]
fn non_semantic_order_changes_do_not_change_graph_identity_property_contract() {
    let a = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"demo"},
          "nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out.txt"}]}],
          "edges":[]
        }"#,
    )
    .expect("parse a");
    let b = parse_graph_strict(
        r#"{
          "meta":{"name":"demo"},
          "edges":[],
          "nodes":[{"outputs":[{"path":"n/out.txt","name":"out"}],"kind":"const","id":"n"}],
          "spec":"bijux-dag/v0.1"
        }"#,
    )
    .expect("parse b");
    assert_eq!(a.graph_id().unwrap(), b.graph_id().unwrap());
}

#[test]
fn semantic_changes_do_change_graph_identity_property_contract() {
    let base = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem"]}],
          "edges":[]
        }"#,
    )
    .expect("parse base");
    let changed = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","printf changed > out"]},"effects":["filesystem"]}],
          "edges":[]
        }"#,
    )
    .expect("parse changed");
    assert_ne!(base.graph_id().unwrap(), changed.graph_id().unwrap());
}

#[test]
fn graph_identity_fixture_families_parse_and_hash() {
    let root = fixtures_root();
    for file in [
        "deep_dependency_tree.json",
        "large_fan_out.json",
        "large_fan_in.json",
        "mixed_shell_container.json",
    ] {
        let graph = parse_fixture(&root.join(file));
        let id = graph.graph_id().expect("graph id");
        assert!(!id.as_str().is_empty(), "graph id must be non-empty for {file}");
    }
}

#[test]
fn canonical_bytes_roundtrip_fixture_corpus() {
    let root = fixtures_root().join("canonical_bytes");
    for file in ["raw_simple.json", "raw_with_defaults.json"] {
        let graph = parse_fixture(&root.join(file));
        let canonical = graph.canonical_json_bytes().expect("canonical bytes");
        let canonical_text = String::from_utf8(canonical).expect("utf8 canonical");
        let reparsed = parse_graph_strict(&canonical_text).expect("reparse canonical");
        assert_eq!(graph.graph_id().unwrap(), reparsed.graph_id().unwrap());
    }
}

#[test]
fn canonical_diff_fixture_corpus_has_expected_drift_before_canonicalization() {
    let root = fixtures_root().join("canonical_diff");
    for file in ["raw_unsorted_env.json", "raw_unsorted_nodes.json"] {
        let raw = std::fs::read_to_string(root.join(file)).expect("read raw");
        let graph = parse_graph_strict(&raw).expect("parse raw");
        let canonical = graph.to_canonical_json().expect("canonical");
        assert_ne!(raw, canonical, "raw and canonical should differ for {file}");
    }
}

#[test]
fn schema_alias_normalization_keeps_identity_stable() {
    let legacy = parse_graph_strict(
        r#"{"spec":"0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out.txt"}]}],"edges":[]}"#,
    )
    .expect("legacy spec parse");
    let modern = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out.txt"}]}],"edges":[]}"#,
    )
    .expect("modern spec parse");
    assert_eq!(legacy.graph_id().unwrap(), modern.graph_id().unwrap());
}

#[test]
fn legacy_spec_aliases_are_accepted_and_normalized() {
    let short = parse_graph_strict(
        r#"{"spec":"0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out.txt"}]}],"edges":[]}"#,
    )
    .expect("0.1 parse");
    let prefixed = parse_graph_strict(
        r#"{"spec":"v0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out.txt"}]}],"edges":[]}"#,
    )
    .expect("v0.1 parse");
    assert_eq!(short.spec, "bijux-dag/v0.1");
    assert_eq!(prefixed.spec, "bijux-dag/v0.1");
}

#[test]
fn default_node_io_fields_normalize_to_same_identity() {
    let implicit = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out.txt"}]}],"edges":[]}"#,
    )
    .expect("implicit io");
    let explicit = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"const","inputs":[],"outputs":[{"name":"out","path":"n/out.txt"}]}],"edges":[]}"#,
    )
    .expect("explicit io");
    assert_eq!(implicit.graph_id().unwrap(), explicit.graph_id().unwrap());
}

#[test]
fn default_resource_values_normalize_to_same_identity() {
    let no_resources = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem"]}],"edges":[]}"#,
    )
    .expect("no resources");
    let explicit_zero = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem"],"resources":{"cpu":0,"mem_mb":0}}],"edges":[]}"#,
    )
    .expect("zero resources");
    assert_eq!(no_resources.graph_id().unwrap(), explicit_zero.graph_id().unwrap());
}

#[test]
fn env_ordering_is_normalized_for_identity() {
    let a = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem","env"],"env_allowlist":["B","A"]}],"edges":[]}"#,
    )
    .expect("env a");
    let b = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem","env"],"env_allowlist":["A","B"]}],"edges":[]}"#,
    )
    .expect("env b");
    assert_eq!(a.graph_id().unwrap(), b.graph_id().unwrap());
}

#[test]
fn graph_identity_does_not_depend_on_backend_adapter_version_env() {
    let graph = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out.txt"}]}],"edges":[]}"#,
    )
    .expect("graph");
    let id_a = graph.graph_id().unwrap();
    let id_b = graph.graph_id().unwrap();
    assert_eq!(id_a, id_b);
}

#[test]
fn execution_affecting_metadata_changes_identity() {
    let base = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem"],"timeout_ms":1000}],"edges":[]}"#,
    )
    .expect("base");
    let changed_timeout = parse_graph_strict(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem"],"timeout_ms":2000}],"edges":[]}"#,
    )
    .expect("changed timeout");
    assert_ne!(base.graph_id().unwrap(), changed_timeout.graph_id().unwrap());
}

#[test]
fn invalid_but_close_canonicalization_fixtures_fail_parse() {
    let root = fixtures_root().join("invalid_close");
    for file in ["invalid_spec_alias_typo.json", "path_traversal_near_valid.json"] {
        let payload = std::fs::read_to_string(root.join(file)).expect("invalid fixture");
        let err = parse_graph_strict(&payload).expect_err("must fail");
        match err {
            GraphError::InvalidSpec(_) | GraphError::ValidationFailed => {}
            other => panic!("unexpected error for {file}: {other:?}"),
        }
    }
}
