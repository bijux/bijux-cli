use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::build_complete_dry_plan_output;
    use crate::{DagBuilder, NodeBuilder, NodeKind};

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
}
