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
use bijux_dag_runtime::{build_plan, RuntimeConfig, Selector, SelectorSet};

mod support;

use support::branch_semantics_graph_json;

fn graph_a() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"a","owners":[],"tags":[]},
      "nodes":[
        {"id":"left","kind":"const","inputs":[],"outputs":[{"name":"out","path":"left/out"}],"params":{"value":1},"tags":["cosmetic"]},
        {"id":"right","kind":"const","inputs":[],"outputs":[{"name":"out","path":"right/out"}],"params":{"value":2}},
        {"id":"join","kind":"shell","inputs":["l","r"],"outputs":[{"name":"out","path":"join/out"}],"params":{"argv":["echo","join"]},"effects":["filesystem"]}
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
        {"id":"join","kind":"shell","inputs":["l","r"],"outputs":[{"name":"out","path":"join/out"}],"params":{"argv":["echo","join"]},"effects":["filesystem"]},
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
    assert_eq!(pa.execution_fingerprint, pb.execution_fingerprint);
    assert_ne!(pa.evidence_fingerprint, pb.evidence_fingerprint);
}

#[test]
fn plan_ordering_is_deterministic() {
    let graph = parse_graph_strict(graph_a()).expect("parse graph");
    let first = build_plan(&graph, &RuntimeConfig::default());
    let second = build_plan(&graph, &RuntimeConfig::default());
    assert_eq!(first.order, second.order);
}

#[test]
fn runtime_plan_preserves_dependency_port_bindings() {
    let graph = parse_graph_strict(graph_a()).expect("parse graph");
    let plan = build_plan(&graph, &RuntimeConfig::default());
    assert_eq!(plan.planned_dependencies.len(), 2);
    let left =
        plan.planned_dependencies.iter().find(|edge| edge.from == "left").expect("left edge");
    assert_eq!(left.from_port, "out");
    assert_eq!(left.to, "join");
    assert_eq!(left.to_port, "l");
}

#[test]
fn runtime_plan_preserves_branch_semantics_and_paths() {
    let graph =
        parse_graph_strict(branch_semantics_graph_json()).expect("parse branch contract graph");

    let plan = build_plan(&graph, &RuntimeConfig::default());
    let branch = plan.planned_nodes.iter().find(|node| node.id == "decide").expect("branch node");
    assert_eq!(branch.semantic_kind, bijux_dag_core::SemanticNodeKind::Branch);
    assert_eq!(branch.executor_kind, "shell");
    assert_eq!(branch.branch.as_ref().expect("branch").decision_output, "decision");

    let conditional = plan
        .planned_dependencies
        .iter()
        .find(|edge| edge.id.as_deref() == Some("branch-left"))
        .expect("conditional edge");
    assert_eq!(conditional.kind, bijux_dag_core::EdgeKind::Conditional);
    assert_eq!(conditional.decision.as_deref(), Some("left"));

    let path = plan
        .branch_paths
        .iter()
        .find(|path| path.branch_node_id == "decide" && path.decision == "left")
        .expect("branch path");
    assert_eq!(path.direct_targets, vec!["left".to_string()]);
    assert!(path.reachable_nodes.contains(&"join".to_string()));
}

#[test]
fn runtime_plan_preserves_node_io_contracts() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "inputs":{"threads":8},
          "nodes":[
            {"id":"seed","kind":"const","inputs":[],"outputs":[{"name":"out","path":"seed/out"}],"params":{"value":1}},
            {
              "id":"run",
              "kind":"shell",
              "inputs":["reads"],
              "outputs":[{"name":"bam","path":"align/out.bam"}],
              "params":{
                "argv":["aligner","--threads",{"graph_input":"threads"}],
                "seed":{"node_output":{"node_id":"seed","output_name":"out"}}
              },
              "effects":["filesystem","env"],
              "env_allowlist":["REFGENOME"]
            }
          ],
          "edges":[
            {"from":{"node_id":"seed","port":"out"},"to":{"node_id":"run","port":"reads"}}
          ]
        }"#,
    )
    .expect("parse graph");

    let plan = build_plan(&graph, &RuntimeConfig::default());
    let run = plan.planned_nodes.iter().find(|node| node.id == "run").expect("run node");
    assert_eq!(run.io_contract.inputs[0].name, "reads");
    assert_eq!(run.io_contract.outputs[0].name, "bam");
    assert_eq!(run.io_contract.env_bindings[0].name, "REFGENOME");
    assert_eq!(run.io_contract.param_bindings.len(), 2);
}

#[test]
fn runtime_plan_preserves_declared_cache_policy() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"fetch",
              "kind":"shell",
              "inputs":[],
              "outputs":[{"name":"out","path":"fetch/out.json"}],
              "params":{"argv":["/bin/sh","-c","date > ../outputs/fetch/out.json"]},
              "cache":{"enabled":false,"reason":"external clock dependency"},
              "effects":["filesystem","clock"]
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("parse graph");

    let plan = build_plan(&graph, &RuntimeConfig::default());
    let fetch = plan.planned_nodes.iter().find(|node| node.id == "fetch").expect("fetch node");
    assert!(!fetch.cache.enabled);
    assert_eq!(fetch.cache.reason.as_deref(), Some("external clock dependency"));
}

#[test]
fn runtime_plan_records_selector_and_closure_provenance() {
    let graph = parse_graph_strict(graph_a()).expect("parse graph");
    let plan = build_plan(
        &graph,
        &RuntimeConfig {
            selectors: SelectorSet {
                include: vec![Selector::IdPrefix("join".to_string())],
                exclude: vec![Selector::Kind("const".to_string())],
            },
            partial_rerun_dependency_closure: true,
            ..RuntimeConfig::default()
        },
    );
    assert_eq!(
        plan.requested_selectors,
        vec!["include:id_prefix:join".to_string(), "exclude:kind:const".to_string()]
    );
    assert!(plan.dependency_closure_enabled);
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
    assert!(plan.order.iter().any(|id| id == "join"));
    assert!(plan.order.iter().any(|id| id == "left"));
    assert!(plan.order.iter().any(|id| id == "right"));
}

#[test]
fn fan_in_fan_out_and_disconnected_lowering_are_supported() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"root","kind":"const","inputs":[],"outputs":[{"name":"out","path":"root/out"}],"params":{"value":1}},
            {"id":"a","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"a/out"}],"params":{"argv":["echo","a"]},"effects":["filesystem"]},
            {"id":"b","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"argv":["echo","b"]},"effects":["filesystem"]},
            {"id":"join","kind":"shell","inputs":["x","y"],"outputs":[{"name":"out","path":"join/out"}],"params":{"argv":["echo","join"]},"effects":["filesystem"]},
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

#[test]
fn runtime_plan_preserves_execution_identity_changes_from_param_updates() {
    let a = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"node","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"node/out"}],"params":{"argv":["echo","one"]},"effects":["filesystem"]}
          ],
          "edges":[]
        }"#,
    )
    .expect("parse a");
    let b = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"node","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"node/out"}],"params":{"argv":["echo","two"]},"effects":["filesystem"]}
          ],
          "edges":[]
        }"#,
    )
    .expect("parse b");

    let pa = build_plan(&a, &RuntimeConfig::default());
    let pb = build_plan(&b, &RuntimeConfig::default());
    assert_eq!(pa.planner_fingerprint, pb.planner_fingerprint);
    assert_ne!(pa.execution_fingerprint, pb.execution_fingerprint);
}
