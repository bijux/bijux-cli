use bijux_dag_runtime as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{build_plan, RuntimeConfig, Selector, SelectorSet};

fn graph_a() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"a","owners":[],"tags":[]},
      "nodes":[
        {"id":"left","kind":"const","inputs":[],"outputs":[{"name":"out","path":"left/out"}],"params":{"value":1},"tags":["cosmetic"]},
        {"id":"right","kind":"const","inputs":[],"outputs":[{"name":"out","path":"right/out"}],"params":{"value":2}},
        {"id":"join","kind":"shell","inputs":["l","r"],"outputs":[{"name":"out","path":"join/out"}],"params":{"argv":["echo","join"]}}
      ],
      "edges":[
        {"from":{"node_id":"left","port":"out"},"to":{"node_id":"join","port":"l"}},
        {"from":{"node_id":"right","port":"out"},"to":{"node_id":"join","port":"r"}}
      ]
    }"#
}

fn graph_b_semantic_equivalent() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"b","description":"cosmetic","owners":[],"tags":[]},
      "nodes":[
        {"id":"join","kind":"shell","inputs":["l","r"],"outputs":[{"name":"out","path":"join/out"}],"params":{"argv":["echo","join"]}},
        {"id":"right","kind":"const","inputs":[],"outputs":[{"name":"out","path":"right/out"}],"params":{"value":2}},
        {"id":"left","kind":"const","inputs":[],"outputs":[{"name":"out","path":"left/out"}],"params":{"value":1},"tags":["changed-only"]}
      ],
      "edges":[
        {"to":{"node_id":"join","port":"r"},"from":{"node_id":"right","port":"out"}},
        {"to":{"node_id":"join","port":"l"},"from":{"node_id":"left","port":"out"}}
      ]
    }"#
}

#[test]
fn semantically_equivalent_graphs_lower_to_same_planner_fingerprint() {
    let a = parse_graph_strict(graph_a()).expect("parse a");
    let b = parse_graph_strict(graph_b_semantic_equivalent()).expect("parse b");
    let pa = build_plan(&a, &RuntimeConfig::default());
    let pb = build_plan(&b, &RuntimeConfig::default());
    assert_eq!(pa.planner_fingerprint, pb.planner_fingerprint);
}

#[test]
fn plan_ordering_is_deterministic() {
    let graph = parse_graph_strict(graph_a()).expect("parse graph");
    let first = build_plan(&graph, &RuntimeConfig::default());
    let second = build_plan(&graph, &RuntimeConfig::default());
    assert_eq!(first.order, second.order);
}

#[test]
fn selector_pruning_stage_is_documented_and_dependency_safe() {
    let graph = parse_graph_strict(graph_a()).expect("parse graph");
    let options = RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::IdPrefix("join".to_string())],
            exclude: vec![],
        },
        ..RuntimeConfig::default()
    };
    let plan = build_plan(&graph, &options);
    assert!(!plan.filter_reasons.contains_key("join"));
    assert!(!plan.filter_reasons.contains_key("left"));
    assert!(!plan.filter_reasons.contains_key("right"));
}

#[test]
fn fan_in_fan_out_and_disconnected_lowering_are_supported() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"root","kind":"const","inputs":[],"outputs":[{"name":"out","path":"root/out"}],"params":{"value":1}},
            {"id":"a","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"a/out"}],"params":{"argv":["echo","a"]}},
            {"id":"b","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"argv":["echo","b"]}},
            {"id":"join","kind":"shell","inputs":["x","y"],"outputs":[{"name":"out","path":"join/out"}],"params":{"argv":["echo","join"]}},
            {"id":"isolated","kind":"const","inputs":[],"outputs":[{"name":"out","path":"isolated/out"}],"params":{"value":9}}
          ],
          "edges":[
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"a","port":"in"}},
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"b","port":"in"}},
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"join","port":"x"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"join","port":"y"}}
          ]
        }"#,
    )
    .expect("parse shapes");

    let plan = build_plan(&graph, &RuntimeConfig::default());
    assert!(plan.planned_nodes.iter().any(|n| n.id == "join"));
    assert!(plan.planned_nodes.iter().any(|n| n.id == "isolated"));
}

#[test]
fn schema_validation_errors_are_distinct_from_planner_errors() {
    let schema_err = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"bad","kind":"const","inputs":[],"outputs":[{"name":"out","path":"../escape"}],"params":{"value":1}}
          ],
          "edges":[]
        }"#,
    )
    .expect_err("invalid output path must fail schema/graph validation");
    assert!(!schema_err.to_string().is_empty());

    let planner_graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"bad-cap","kind":"const","inputs":[],"outputs":[{"name":"out","path":"bad/out"}],"resources":{"cpu":1,"mem_mb":64},"params":{"value":1}}
          ],
          "edges":[]
        }"#,
    )
    .expect("graph should parse");
    let plan = build_plan(&planner_graph, &RuntimeConfig::default());
    assert!(plan
        .diagnostics
        .iter()
        .any(|d| d.contains("P4021") && d.contains("unsupported-runtime-capability")));
}
