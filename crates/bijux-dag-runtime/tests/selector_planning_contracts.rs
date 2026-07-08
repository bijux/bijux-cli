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
use bijux_dag_runtime::{
    build_planner_analysis, PlannerGuardrails, RuntimeConfig, Selector, SelectorSet,
};

fn selector_graph() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "nodes":[
        {"id":"prep","kind":"const","outputs":[{"name":"out","path":"prep/out"}],"params":{"value":1},"tags":["seed"]},
        {"id":"prep-archive","kind":"const","outputs":[{"name":"out","path":"prep-archive/out"}],"params":{"value":2},"tags":["archive"]},
        {"id":"train","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"train/out"}],"params":{"argv":["echo","train"]},"effects":["filesystem"],"tags":["ml"]},
        {"id":"report","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"report/out"}],"params":{"argv":["echo","report"]},"effects":["filesystem"],"tags":["publish"]}
      ],
      "edges":[
        {"from":{"node_id":"prep","port":"out"},"to":{"node_id":"train","port":"in"}},
        {"from":{"node_id":"train","port":"out"},"to":{"node_id":"report","port":"in"}}
      ]
    }"#
}

fn planner_result(options: RuntimeConfig) -> bijux_dag_runtime::PlannerBuildResult {
    let graph = parse_graph_strict(selector_graph()).expect("graph");
    build_planner_analysis(
        &graph,
        &options,
        &options.selectors,
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner analysis")
}

#[test]
fn exact_id_selector_does_not_match_prefix_peers() {
    let result = planner_result(RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::Id("prep".to_string())],
            exclude: Vec::new(),
        },
        ..RuntimeConfig::default()
    });

    assert!(!result.plan.filter_reasons.contains_key("prep"));
    assert_eq!(
        result.plan.filter_reasons.get("prep-archive").map(String::as_str),
        Some("not_selected_by_include_selector")
    );
}

#[test]
fn explicit_prefix_selector_can_match_multiple_nodes() {
    let result = planner_result(RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::IdPrefix("prep".to_string())],
            exclude: Vec::new(),
        },
        ..RuntimeConfig::default()
    });

    assert!(!result.plan.filter_reasons.contains_key("prep"));
    assert!(!result.plan.filter_reasons.contains_key("prep-archive"));
}

#[test]
fn tag_selector_filters_to_tagged_nodes() {
    let result = planner_result(RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::Tag("ml".to_string())],
            exclude: Vec::new(),
        },
        partial_rerun_dependency_closure: false,
        ..RuntimeConfig::default()
    });

    assert!(!result.plan.filter_reasons.contains_key("train"));
    assert_eq!(
        result.plan.filter_reasons.get("prep").map(String::as_str),
        Some("not_selected_by_include_selector")
    );
}

#[test]
fn kind_selector_and_exclude_selector_can_be_combined() {
    let result = planner_result(RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::Kind("shell".to_string())],
            exclude: vec![Selector::Id("report".to_string())],
        },
        ..RuntimeConfig::default()
    });

    assert!(!result.plan.filter_reasons.contains_key("train"));
    assert_eq!(
        result.plan.filter_reasons.get("report").map(String::as_str),
        Some("excluded_by_selector")
    );
}

#[test]
fn dependency_closure_annotations_distinguish_direct_and_required_nodes() {
    let result = planner_result(RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::Id("report".to_string())],
            exclude: Vec::new(),
        },
        partial_rerun_dependency_closure: true,
        ..RuntimeConfig::default()
    });

    let prep = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "prep")
        .expect("prep annotation");
    let report = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "report")
        .expect("report annotation");

    assert_eq!(prep.reason, "selected_by_dependency_closure");
    assert_eq!(report.reason, "selected_by_include_selector");
    assert!(prep.selected);
    assert!(report.selected);
}

#[test]
fn downstream_roots_select_exact_root_and_descendants() {
    let result = planner_result(RuntimeConfig {
        downstream_selection_roots: vec!["train".to_string()],
        partial_rerun_dependency_closure: false,
        ..RuntimeConfig::default()
    });

    let train = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "train")
        .expect("train annotation");
    let report = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "report")
        .expect("report annotation");
    let prep = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "prep")
        .expect("prep annotation");

    assert_eq!(train.reason, "selected_by_from_node");
    assert_eq!(report.reason, "selected_by_downstream_closure");
    assert_eq!(prep.reason, "not_selected_by_from_node");
    assert!(train.selected);
    assert!(report.selected);
    assert!(!prep.selected);
    assert_eq!(result.plan.requested_selectors, vec!["from-node:train"]);
}

#[test]
fn upstream_targets_select_exact_target_and_required_ancestors() {
    let result = planner_result(RuntimeConfig {
        upstream_selection_targets: vec!["report".to_string()],
        partial_rerun_dependency_closure: false,
        ..RuntimeConfig::default()
    });

    let prep = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "prep")
        .expect("prep annotation");
    let train = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "train")
        .expect("train annotation");
    let report = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "report")
        .expect("report annotation");
    let prep_archive = result
        .annotations
        .iter()
        .find(|annotation| annotation.node_id == "prep-archive")
        .expect("prep-archive annotation");

    assert_eq!(prep.reason, "selected_by_upstream_closure");
    assert_eq!(train.reason, "selected_by_upstream_closure");
    assert_eq!(report.reason, "selected_by_to_node");
    assert_eq!(prep_archive.reason, "not_selected_by_to_node");
    assert!(prep.selected);
    assert!(train.selected);
    assert!(report.selected);
    assert!(!prep_archive.selected);
    assert_eq!(result.plan.requested_selectors, vec!["to-node:report"]);
}
