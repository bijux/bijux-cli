use bijux_dag_core::Graph;
use bijux_dag_runtime::{PlannerBuildResult, RunSnapshot};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OmittedNodeSummary {
    pub(crate) node_id: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SelectionSummary {
    pub(crate) requested_selectors: Vec<String>,
    pub(crate) upstream_targets: Vec<String>,
    pub(crate) downstream_roots: Vec<String>,
    pub(crate) dependency_closure_enabled: bool,
    pub(crate) selected_nodes: Vec<String>,
    pub(crate) omitted_nodes: Vec<OmittedNodeSummary>,
}

pub(crate) fn selection_summary_from_planner(result: &PlannerBuildResult) -> SelectionSummary {
    let mut selected_nodes = result
        .annotations
        .iter()
        .filter(|annotation| annotation.selected)
        .map(|annotation| annotation.node_id.clone())
        .collect::<Vec<_>>();
    let mut omitted_nodes = result
        .annotations
        .iter()
        .filter(|annotation| !annotation.selected)
        .map(|annotation| OmittedNodeSummary {
            node_id: annotation.node_id.clone(),
            reason: annotation.reason.clone(),
        })
        .collect::<Vec<_>>();

    selected_nodes.sort();
    omitted_nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));

    SelectionSummary {
        requested_selectors: result.plan.requested_selectors.clone(),
        upstream_targets: result
            .plan
            .requested_selectors
            .iter()
            .filter_map(|value| value.strip_prefix("to-node:").map(str::to_string))
            .collect(),
        downstream_roots: result
            .plan
            .requested_selectors
            .iter()
            .filter_map(|value| value.strip_prefix("from-node:").map(str::to_string))
            .collect(),
        dependency_closure_enabled: result.plan.dependency_closure_enabled,
        selected_nodes,
        omitted_nodes,
    }
}

pub(crate) fn selection_summary_for_all_nodes(graph: &Graph) -> SelectionSummary {
    let mut selected_nodes = graph.nodes.iter().map(|node| node.id.clone()).collect::<Vec<_>>();
    selected_nodes.sort();
    SelectionSummary {
        requested_selectors: Vec::new(),
        upstream_targets: Vec::new(),
        downstream_roots: Vec::new(),
        dependency_closure_enabled: false,
        selected_nodes,
        omitted_nodes: Vec::new(),
    }
}

pub(crate) fn selection_summary_from_run_snapshot(
    graph: &Graph,
    snapshot: &RunSnapshot,
) -> SelectionSummary {
    let selected = snapshot.selected_nodes.iter().cloned().collect::<BTreeSet<_>>();
    let mut selected_nodes = selected.iter().cloned().collect::<Vec<_>>();
    let mut omitted_nodes = graph
        .nodes
        .iter()
        .filter(|node| !selected.contains(&node.id))
        .map(|node| OmittedNodeSummary {
            node_id: node.id.clone(),
            reason: "omitted_from_run_snapshot".to_string(),
        })
        .collect::<Vec<_>>();
    selected_nodes.sort();
    omitted_nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));

    SelectionSummary {
        requested_selectors: snapshot.requested_selectors.clone(),
        upstream_targets: snapshot
            .requested_selectors
            .iter()
            .filter_map(|value| value.strip_prefix("to-node:").map(str::to_string))
            .collect(),
        downstream_roots: snapshot
            .requested_selectors
            .iter()
            .filter_map(|value| value.strip_prefix("from-node:").map(str::to_string))
            .collect(),
        dependency_closure_enabled: snapshot.dependency_closure_enabled,
        selected_nodes,
        omitted_nodes,
    }
}

#[cfg(test)]
mod tests {
    use super::selection_summary_from_planner;
    use crate::routes::plan_routes::PlanPreviewConfig;

    #[test]
    fn selection_summary_preserves_requested_selectors_and_reasons() {
        let graph = crate::parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[
                {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
                {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}},
                {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":3}}
              ],
              "edges":[
                {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
                {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in"}}
              ]
            }"#,
        )
        .expect("graph");
        let preview = PlanPreviewConfig {
            selectors: bijux_dag_runtime::SelectorSet {
                include: vec![bijux_dag_runtime::Selector::Id("b".to_string())],
                exclude: Vec::new(),
            },
            dependency_closure: true,
            ..PlanPreviewConfig::default()
        };
        let result = crate::routes::plan_routes::build_default_planner_analysis(&graph, &preview)
            .expect("planner analysis");

        let summary = selection_summary_from_planner(&result);

        assert_eq!(summary.requested_selectors, vec!["include:id:b".to_string()]);
        assert_eq!(summary.selected_nodes, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            summary.omitted_nodes,
            vec![super::OmittedNodeSummary {
                node_id: "c".to_string(),
                reason: "not_selected_by_include_selector".to_string(),
            }]
        );
        assert!(summary.dependency_closure_enabled);
    }
}
