use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{parse_graph_strict, ParamValue, RetryPolicy, SPEC_VERSION};
use serde_json::json;

fn parse(text: &str) -> bijux_dag_core::Graph {
    parse_graph_strict(text).expect("parse")
}

#[test]
fn parse_rejects_unknown_fields_fixture() {
    let payload =
        include_str!("../../../configs/dag/schema/fixtures/v0.1/negative/unknown-field.json");
    assert!(parse_graph_strict(payload).is_err());
}

#[test]
fn parse_rejects_future_required_behavior_fixture() {
    let payload = include_str!(
        "../../../configs/dag/schema/fixtures/v0.1/negative/future-required-behavior.json"
    );
    assert!(parse_graph_strict(payload).is_err());
}

#[test]
fn parse_rejects_ambiguous_output_paths_fixture() {
    let payload =
        include_str!("../../../configs/dag/schema/fixtures/v0.1/negative/invalid-output-path.json");
    assert!(parse_graph_strict(payload).is_err());
}

#[test]
fn canonical_graph_shape_coverage() {
    let fixtures = [
        include_str!("../../../configs/dag/schema/fixtures/v0.1/positive/empty-graph.json"),
        include_str!("../../../configs/dag/schema/fixtures/v0.1/positive/isolated-node.json"),
        include_str!("../../../configs/dag/schema/fixtures/v0.1/positive/diamond.json"),
        include_str!("../../../configs/dag/schema/fixtures/v0.1/positive/fan-in.json"),
        include_str!("../../../configs/dag/schema/fixtures/v0.1/positive/fan-out.json"),
        include_str!("../../../configs/dag/schema/fixtures/v0.1/positive/disconnected-groups.json"),
    ];

    for fixture in fixtures {
        let graph = parse(fixture);
        assert_eq!(
            graph.to_canonical_json().expect("canonical"),
            graph.to_canonical_json().expect("canonical")
        );
    }
}

#[test]
fn diagnostics_order_and_message_stability() {
    let mut graph = parse(&format!(
        r#"{{
  "spec": "{}",
  "meta": {{"name": "bad graph", "tags": ["good", "bad tag"]}},
  "nodes": [
    {{"id":"a","kind":"shell","inputs":[],"outputs":[{{"name":"out","path":"a/out"}}],"tags":["bad tag"],"params":{{"argv":["echo","ok"]}}}}
  ],
  "edges": []
}}"#,
        SPEC_VERSION
    ));
    graph.nodes[0].retry = RetryPolicy { max_attempts: 1, backoff_ms: 0 };
    graph.nodes[0].effects = vec![];
    graph.nodes[0].params = ParamValue::Literal(json!({"argv":["echo","ok"]}));

    let diags = graph.validate_with_warnings();
    let snapshot = diags
        .iter()
        .map(|d| format!("{}|{}|{}", d.code, d.path, d.message))
        .collect::<Vec<_>>()
        .join("\n");

    let expected = [
        "E1009|/nodes/a/effects|missing effects for shell node: a",
        "E1009|/nodes/a/effects|shell node missing filesystem effect: a",
        "E1026|/meta/tags|illegal graph tag: bad tag",
        "E1026|/nodes/a/tags|illegal node tag: bad tag",
        "E1027|/meta/name|illegal graph name: bad graph",
        "W2002|/nodes/a|orphan node: a",
    ]
    .join("\n");

    assert_eq!(snapshot, expected);
}

#[test]
fn namespaced_tags_remain_valid_for_scheduler_routing_contracts() {
    let graph = parse(&format!(
        r#"{{
  "spec": "{}",
  "meta": {{"name": "graph", "tags": ["release.candidate:2026"]}},
  "nodes": [
    {{
      "id":"render",
      "kind":"shell",
      "inputs":[],
      "outputs":[{{"name":"out","path":"render/out"}}],
      "tags":["slurm.partition:gpu","slurm.queue:priority","team_render"],
      "effects":["filesystem"],
      "params":{{"argv":["echo","ok"]}}
    }}
  ],
  "edges": []
}}"#,
        SPEC_VERSION
    ));

    let diagnostics = graph.validate_with_warnings();
    assert!(!diagnostics.iter().any(|diagnostic| diagnostic.code == "E1026"));
}

#[test]
fn canonicalization_stable_across_path_separator_variants() {
    let a = parse(&format!(
        r#"{{
  "spec": "{}",
  "nodes": [{{"id":"a","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"dir\\file.txt"}}]}}],
  "edges": []
}}"#,
        SPEC_VERSION
    ));
    let b = parse(&format!(
        r#"{{
  "spec": "{}",
  "nodes": [{{"id":"a","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"dir/file.txt"}}]}}],
  "edges": []
}}"#,
        SPEC_VERSION
    ));

    assert_eq!(
        a.to_canonical_json().expect("canonical"),
        b.to_canonical_json().expect("canonical")
    );
    assert_eq!(a.graph_fingerprint().expect("fp"), b.graph_fingerprint().expect("fp"));
}
