use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{
    parse_graph_strict,
    validate::{
        validate_graph, validate_schema, validate_semantics, validate_topology,
        validation_rule_registry,
    },
};

#[test]
fn validate_module_entrypoints_are_covered_and_consistent() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"validate-entrypoints","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("graph");

    let full = validate_graph(&graph);
    assert!(
        full.is_empty(),
        "expected no validation diagnostics for valid graph"
    );
    assert_eq!(validate_schema(&graph).len(), full.len());
    assert_eq!(validate_semantics(&graph).len(), full.len());
    assert_eq!(validate_topology(&graph).len(), full.len());

    let rules = validation_rule_registry();
    assert!(
        !rules.is_empty(),
        "validation rule registry must not be empty"
    );
    assert!(rules.iter().any(|r| r.id == "E1025"));
}
