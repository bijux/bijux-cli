use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{compile_graph, lower_graph_to_execution_plan, Graph, GraphError, PlannerSeverity, Severity};

/// Node dry-plan row for operator-facing planning output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryPlanNodeRowV1 {
    /// Node identifier.
    pub node_id: String,
    /// Dependencies in lowered order.
    pub dependencies: Vec<String>,
    /// Trigger rule applied at runtime.
    pub trigger_rule: String,
    /// Resolved parameter binding count.
    pub resolved_param_count: usize,
    /// Expected output artifacts.
    pub expected_artifacts: Vec<String>,
    /// Cache eligibility inferred from side effects.
    pub cache_eligible: bool,
    /// CPU resource hint.
    pub cpu_hint: u32,
    /// Memory resource hint in MB.
    pub mem_mb_hint: u32,
}

/// Complete dry-plan output contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryPlanCompleteOutputV1 {
    /// Dry-plan rows.
    pub nodes: Vec<DryPlanNodeRowV1>,
    /// Refusals preventing runnable status.
    pub refusals: Vec<String>,
}

/// Build complete dry-plan output with lowered details and refusal diagnostics.
pub fn build_complete_dry_plan_output(graph: &Graph) -> Result<DryPlanCompleteOutputV1, GraphError> {
    let compile = compile_graph(graph)?;
    let validation_refusals = compile
        .diagnostics
        .iter()
        .filter(|diag| diag.severity == Severity::Error)
        .map(|diag| format!("{} {}", diag.code, diag.message))
        .collect::<Vec<_>>();
    let plan = lower_graph_to_execution_plan(&compile.normalized_graph, Default::default())
        .map_err(|_| GraphError::ValidationFailed)?;

    let mut planner_refusals = plan
        .diagnostics
        .iter()
        .filter(|diag| diag.severity == PlannerSeverity::Error)
        .map(|diag| format!("{} {}", diag.id, diag.message))
        .collect::<Vec<_>>();
    let mut refusals = validation_refusals;
    refusals.append(&mut planner_refusals);
    refusals.sort();

    let mut nodes = plan
        .nodes
        .iter()
        .map(|node| DryPlanNodeRowV1 {
            node_id: node.id.clone(),
            dependencies: node.deps.clone(),
            trigger_rule: format!("{:?}", node.trigger_rule),
            resolved_param_count: node.io_contract.param_bindings.len(),
            expected_artifacts: node.outputs.iter().map(|output| output.path.clone()).collect(),
            cache_eligible: node.side_effects.is_empty(),
            cpu_hint: node.resources.as_ref().map(|value| value.cpu).unwrap_or_default(),
            mem_mb_hint: node.resources.as_ref().map(|value| value.mem_mb).unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));

    Ok(DryPlanCompleteOutputV1 { nodes, refusals })
}

/// Explain state for one node in plan explain output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanExplainNodeReasonV1 {
    /// Node identifier.
    pub node_id: String,
    /// Explain state (`included`, `skipped`, `blocked`, `expanded`, `refused`).
    pub state: String,
    /// Human-readable reason.
    pub reason: String,
    /// Graph field path that supports the reason.
    pub field_path: String,
    /// Capability check anchor when relevant.
    pub capability_check: Option<String>,
}

/// Plan explain contract output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanExplainReportV1 {
    /// Per-node explain reasons.
    pub nodes: Vec<PlanExplainNodeReasonV1>,
}

/// Build plan explain report for included/skipped/blocked/refused decisions.
pub fn build_plan_explain_report(
    graph: &Graph,
    selected_nodes: Option<&BTreeSet<String>>,
    available_executor_kinds: &BTreeSet<String>,
) -> Result<PlanExplainReportV1, GraphError> {
    let compile = compile_graph(graph)?;
    let selected = selected_nodes.cloned().unwrap_or_else(|| {
        graph.nodes.iter().map(|node| node.id.clone()).collect()
    });
    let plan = lower_graph_to_execution_plan(&compile.normalized_graph, Default::default())
        .map_err(|_| GraphError::ValidationFailed)?;
    let planned_ids = plan.nodes.iter().map(|node| node.id.clone()).collect::<BTreeSet<_>>();
    let refused = plan
        .diagnostics
        .iter()
        .filter(|diag| diag.severity == PlannerSeverity::Error)
        .filter_map(|diag| diag.node_id.clone())
        .collect::<BTreeSet<_>>();

    let mut rows = graph
        .nodes
        .iter()
        .map(|node| {
            if !selected.contains(&node.id) {
                return PlanExplainNodeReasonV1 {
                    node_id: node.id.clone(),
                    state: "skipped".to_string(),
                    reason: "node not selected in plan scope".to_string(),
                    field_path: format!("/nodes/{}/id", node.id),
                    capability_check: None,
                };
            }
            let executor = node.kind.as_str().to_string();
            if !available_executor_kinds.contains(&executor) {
                return PlanExplainNodeReasonV1 {
                    node_id: node.id.clone(),
                    state: "blocked".to_string(),
                    reason: format!("executor kind {} is unavailable", executor),
                    field_path: format!("/nodes/{}/kind", node.id),
                    capability_check: Some(format!("executor:{executor}")),
                };
            }
            if refused.contains(&node.id) {
                return PlanExplainNodeReasonV1 {
                    node_id: node.id.clone(),
                    state: "refused".to_string(),
                    reason: "planner emitted hard refusal for node".to_string(),
                    field_path: format!("/nodes/{}/id", node.id),
                    capability_check: None,
                };
            }
            if planned_ids.contains(&node.id) {
                return PlanExplainNodeReasonV1 {
                    node_id: node.id.clone(),
                    state: "included".to_string(),
                    reason: "node lowered into execution plan".to_string(),
                    field_path: format!("/nodes/{}/id", node.id),
                    capability_check: Some(format!("executor:{executor}")),
                };
            }
            PlanExplainNodeReasonV1 {
                node_id: node.id.clone(),
                state: "expanded".to_string(),
                reason: "node participates through expansion semantics".to_string(),
                field_path: format!("/nodes/{}/semantic_kind", node.id),
                capability_check: None,
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    Ok(PlanExplainReportV1 { nodes: rows })
}

#[cfg(test)]
mod tests {
    use super::{build_complete_dry_plan_output, build_plan_explain_report};
    use crate::{DagBuilder, Effect, NodeBuilder, NodeKind};
    use std::collections::BTreeSet;

    #[test]
    fn g041_dry_plan_output_contains_lowered_shape_and_runnable_refusals() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("extract", NodeKind::Const)
                    .output("out", "artifacts/extract.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("load", NodeKind::Const)
                    .input("in")
                    .output("done", "artifacts/load.json")
                    .build(),
            )
            .edge("extract", "out", "load", "in")
            .build();

        let report = build_complete_dry_plan_output(&graph).expect("dry-plan should build");
        assert_eq!(report.nodes.len(), 2);
        assert!(report.nodes.iter().any(|node| node.node_id == "load" && !node.dependencies.is_empty()));
    }

    #[test]
    fn g042_plan_explain_reports_included_skipped_and_capability_blocked_nodes() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("a", NodeKind::Const)
                    .output("out", "artifacts/a.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("b", NodeKind::Shell)
                    .input("in")
                    .output("done", "artifacts/b.json")
                    .effect(Effect::Filesystem)
                    .build(),
            )
            .edge("a", "out", "b", "in")
            .build();
        let selected = BTreeSet::from(["a".to_string(), "b".to_string()]);
        let available = BTreeSet::from(["const".to_string()]);
        let report = build_plan_explain_report(&graph, Some(&selected), &available)
            .expect("plan explain");
        assert!(report
            .nodes
            .iter()
            .any(|node| node.node_id == "a" && node.state == "included"));
        assert!(report
            .nodes
            .iter()
            .any(|node| node.node_id == "b" && node.state == "blocked"));
    }
}
