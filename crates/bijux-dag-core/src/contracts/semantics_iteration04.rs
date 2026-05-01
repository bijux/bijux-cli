use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{lower_graph_to_execution_plan, EdgeKind, Graph, ParamValue, SemanticNodeKind};

/// Branch decision evidence artifact with replay identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchDecisionArtifactV1 {
    /// Branch node identifier.
    pub branch_node_id: String,
    /// Serialized predicate input used for decisioning.
    pub predicate_input: String,
    /// Selected decision label.
    pub chosen_branch: String,
    /// Branches not selected.
    pub skipped_branches: Vec<String>,
    /// Stable replay identity for this decision event.
    pub replay_identity: String,
}

/// Build a first-class branch decision artifact.
pub fn build_branch_decision_artifact(
    branch_node_id: &str,
    predicate_input: &str,
    chosen_branch: &str,
    declared_branches: &[String],
    replay_identity: &str,
) -> Result<BranchDecisionArtifactV1, String> {
    for (name, value) in [
        ("branch_node_id", branch_node_id),
        ("predicate_input", predicate_input),
        ("chosen_branch", chosen_branch),
        ("replay_identity", replay_identity),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{name} cannot be empty"));
        }
    }
    if !declared_branches.iter().any(|branch| branch == chosen_branch) {
        return Err("chosen_branch must be declared".to_string());
    }
    let mut skipped_branches = declared_branches
        .iter()
        .filter(|branch| branch.as_str() != chosen_branch)
        .cloned()
        .collect::<Vec<_>>();
    skipped_branches.sort();

    Ok(BranchDecisionArtifactV1 {
        branch_node_id: branch_node_id.to_string(),
        predicate_input: predicate_input.to_string(),
        chosen_branch: chosen_branch.to_string(),
        skipped_branches,
        replay_identity: replay_identity.to_string(),
    })
}

/// Explicit upstream terminal states for trigger-rule reasoning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamTerminalStateV1 {
    Success,
    Failed,
    Skipped,
    Cancelled,
}

/// Per-node terminal state summary captured in run evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeTerminalStateRecordV1 {
    /// Node identifier.
    pub node_id: String,
    /// Final terminal state.
    pub state: UpstreamTerminalStateV1,
    /// Whether an execution attempt happened.
    pub executed: bool,
}

/// Trigger readiness derived from explicit parent states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerReadinessFromStatesV1 {
    /// Trigger rule identifier.
    pub trigger_rule: String,
    /// Whether node is runnable.
    pub runnable: bool,
    /// Explanation for decision.
    pub reason: String,
}

/// Evaluate trigger readiness with skipped state modeled explicitly.
pub fn evaluate_trigger_readiness_from_states(
    trigger_rule: &str,
    parent_states: &[UpstreamTerminalStateV1],
) -> Result<TriggerReadinessFromStatesV1, String> {
    if parent_states.is_empty() {
        return Ok(TriggerReadinessFromStatesV1 {
            trigger_rule: trigger_rule.to_string(),
            runnable: true,
            reason: "no parents".to_string(),
        });
    }
    let success = parent_states
        .iter()
        .filter(|state| **state == UpstreamTerminalStateV1::Success)
        .count();
    let failed = parent_states
        .iter()
        .filter(|state| **state == UpstreamTerminalStateV1::Failed)
        .count();
    let cancelled = parent_states
        .iter()
        .filter(|state| **state == UpstreamTerminalStateV1::Cancelled)
        .count();
    let total = parent_states.len();

    let result = match trigger_rule {
        "all_success" => (
            success == total,
            "requires every parent in success and treats skipped as non-success",
        ),
        "all_done" => (success + failed + cancelled <= total, "all terminal states accepted"),
        "any_success" => (success > 0, "requires at least one successful parent"),
        "none_failed" => (failed == 0, "requires zero failed parents"),
        _ => return Err("unsupported trigger_rule".to_string()),
    };

    Ok(TriggerReadinessFromStatesV1 {
        trigger_rule: trigger_rule.to_string(),
        runnable: result.0,
        reason: result.1.to_string(),
    })
}

/// Barrier semantics violation detected during graph authoring validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarrierSemanticViolationV1 {
    /// Stable rule code.
    pub code: String,
    /// Node identifier.
    pub node_id: String,
    /// Remediation guidance.
    pub remediation: String,
}

/// Barrier semantic validation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarrierSemanticReportV1 {
    /// Whether all barrier nodes are semantically valid.
    pub valid: bool,
    /// Violations found.
    pub violations: Vec<BarrierSemanticViolationV1>,
}

/// Validate barrier semantics before runtime execution.
pub fn validate_barrier_semantics(graph: &Graph) -> BarrierSemanticReportV1 {
    let mut violations = Vec::new();
    for node in &graph.nodes {
        if node.semantic_kind != SemanticNodeKind::Barrier {
            continue;
        }
        if node.inputs.is_empty() {
            violations.push(BarrierSemanticViolationV1 {
                code: "B3301_BARRIER_REQUIRES_INPUTS".to_string(),
                node_id: node.id.clone(),
                remediation: "connect barrier node to one or more upstream dependencies".to_string(),
            });
        }
        if !node.outputs.is_empty() {
            violations.push(BarrierSemanticViolationV1 {
                code: "B3302_BARRIER_MUST_NOT_DECLARE_OUTPUTS".to_string(),
                node_id: node.id.clone(),
                remediation: "remove data outputs from barrier nodes; barriers synchronize only"
                    .to_string(),
            });
        }
        if !matches!(node.params, ParamValue::Literal(Value::Null)) {
            violations.push(BarrierSemanticViolationV1 {
                code: "B3303_BARRIER_MUST_NOT_MUTATE_PARAMS".to_string(),
                node_id: node.id.clone(),
                remediation: "drop params from barrier node; do not model transforms on barriers"
                    .to_string(),
            });
        }
    }
    violations.sort_by(|left, right| left.code.cmp(&right.code).then_with(|| left.node_id.cmp(&right.node_id)));
    BarrierSemanticReportV1 { valid: violations.is_empty(), violations }
}

/// Reducer input ordering policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReducerOrderingPolicyV1 {
    Topological,
    LexicographicNodeId,
}

/// Reducer semantic violation detected during validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReducerSemanticViolationV1 {
    /// Stable violation code.
    pub code: String,
    /// Reducer node id.
    pub node_id: String,
    /// Remediation guidance.
    pub remediation: String,
}

/// Reducer semantic report including deterministic upstream ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReducerSemanticReportV1 {
    /// Whether report passed reducer checks.
    pub valid: bool,
    /// Upstream order by reducer node.
    pub upstream_order: Vec<(String, Vec<String>)>,
    /// Violations found.
    pub violations: Vec<ReducerSemanticViolationV1>,
}

/// Validate reducer semantics and produce deterministic upstream ordering.
pub fn validate_reducer_semantics(
    graph: &Graph,
    ordering_policy: ReducerOrderingPolicyV1,
    allow_empty_collection: bool,
) -> ReducerSemanticReportV1 {
    let mut upstream_order = Vec::new();
    let mut violations = Vec::new();
    for node in &graph.nodes {
        if node.semantic_kind != SemanticNodeKind::Reduce {
            continue;
        }
        if node.outputs.len() != 1 {
            violations.push(ReducerSemanticViolationV1 {
                code: "R3401_REDUCER_REQUIRES_SINGLE_OUTPUT".to_string(),
                node_id: node.id.clone(),
                remediation: "declare exactly one reducer output artifact".to_string(),
            });
        }
        let mut upstreams = graph
            .edges
            .iter()
            .filter(|edge| edge.to.node_id == node.id)
            .map(|edge| edge.from.node_id.clone())
            .collect::<Vec<_>>();
        if upstreams.is_empty() && !allow_empty_collection {
            violations.push(ReducerSemanticViolationV1 {
                code: "R3402_REDUCER_EMPTY_COLLECTION_FORBIDDEN".to_string(),
                node_id: node.id.clone(),
                remediation: "connect at least one upstream producer or enable empty policy"
                    .to_string(),
            });
        }

        let mut target_ports = std::collections::BTreeSet::new();
        for edge in graph.edges.iter().filter(|edge| edge.to.node_id == node.id) {
            let port_key = edge.to.port.clone();
            if !target_ports.insert(port_key) {
                violations.push(ReducerSemanticViolationV1 {
                    code: "R3403_REDUCER_AMBIGUOUS_FANIN".to_string(),
                    node_id: node.id.clone(),
                    remediation: "bind each reducer input port from exactly one upstream edge"
                        .to_string(),
                });
                break;
            }
        }

        match ordering_policy {
            ReducerOrderingPolicyV1::Topological => {}
            ReducerOrderingPolicyV1::LexicographicNodeId => upstreams.sort(),
        }
        upstream_order.push((node.id.clone(), upstreams));
    }
    upstream_order.sort_by(|left, right| left.0.cmp(&right.0));
    violations.sort_by(|left, right| left.code.cmp(&right.code).then_with(|| left.node_id.cmp(&right.node_id)));
    ReducerSemanticReportV1 { valid: violations.is_empty(), upstream_order, violations }
}

/// Edge semantics record from lowered execution plans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeSemanticRecordV1 {
    /// Edge kind.
    pub kind: String,
    /// Source node.
    pub from: String,
    /// Destination node.
    pub to: String,
    /// Conditional decision label when present.
    pub decision: Option<String>,
}

/// Edge semantics snapshot that proves lowering preserves edge kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeSemanticsSnapshotV1 {
    /// Edge count by kind.
    pub counts: Vec<(String, usize)>,
    /// Lowered edge records.
    pub lowered_edges: Vec<EdgeSemanticRecordV1>,
}

/// Build edge semantics snapshot from graph and lowered plan.
pub fn build_edge_semantics_snapshot(graph: &Graph) -> Result<EdgeSemanticsSnapshotV1, String> {
    let plan =
        lower_graph_to_execution_plan(graph, Default::default()).map_err(|err| err.to_string())?;
    let mut counts = vec![
        (
            "data".to_string(),
            graph.edges.iter().filter(|edge| edge.kind == EdgeKind::Data).count(),
        ),
        (
            "control".to_string(),
            graph.edges.iter().filter(|edge| edge.kind == EdgeKind::Control).count(),
        ),
        (
            "conditional".to_string(),
            graph.edges.iter().filter(|edge| edge.kind == EdgeKind::Conditional).count(),
        ),
    ];
    counts.sort_by(|left, right| left.0.cmp(&right.0));
    let lowered_edges = plan
        .edges
        .into_iter()
        .map(|edge| EdgeSemanticRecordV1 {
            kind: match edge.kind {
                EdgeKind::Data => "data".to_string(),
                EdgeKind::Control => "control".to_string(),
                EdgeKind::Conditional => "conditional".to_string(),
            },
            from: edge.from,
            to: edge.to,
            decision: edge.decision,
        })
        .collect::<Vec<_>>();
    Ok(EdgeSemanticsSnapshotV1 { counts, lowered_edges })
}

#[cfg(test)]
mod tests {
    use super::{
        build_branch_decision_artifact, build_edge_semantics_snapshot,
        evaluate_trigger_readiness_from_states, validate_barrier_semantics,
        validate_reducer_semantics, ReducerOrderingPolicyV1, UpstreamTerminalStateV1,
    };
    use crate::{BranchSpec, DagBuilder, EdgeKind, NodeBuilder, NodeKind, SemanticNodeKind, TriggerRule};

    #[test]
    fn g031_branch_decision_artifact_persists_chosen_and_skipped_branches() {
        let artifact = build_branch_decision_artifact(
            "branch.qc",
            r#"{"metric":"coverage","value":0.91}"#,
            "pass",
            &["pass".to_string(), "fail".to_string()],
            "run=demo;node=branch.qc;decision=pass",
        )
        .expect("artifact should build");
        assert_eq!(artifact.chosen_branch, "pass");
        assert_eq!(artifact.skipped_branches, vec!["fail".to_string()]);
    }

    #[test]
    fn g032_skipped_state_is_explicit_and_affects_trigger_readiness() {
        let readiness = evaluate_trigger_readiness_from_states(
            "all_success",
            &[UpstreamTerminalStateV1::Success, UpstreamTerminalStateV1::Skipped],
        )
        .expect("trigger readiness should evaluate");
        assert!(!readiness.runnable);
        assert!(readiness.reason.contains("skipped"));
    }

    #[test]
    fn g033_barrier_semantics_refuse_invalid_usage_pre_runtime() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("barrier", NodeKind::Const)
                    .semantic_kind(SemanticNodeKind::Barrier)
                    .output("out", "artifacts/not-allowed.json")
                    .build(),
            )
            .build();
        let report = validate_barrier_semantics(&graph);
        assert!(!report.valid);
        assert!(report
            .violations
            .iter()
            .any(|item| item.code == "B3301_BARRIER_REQUIRES_INPUTS"));
        assert!(report
            .violations
            .iter()
            .any(|item| item.code == "B3302_BARRIER_MUST_NOT_DECLARE_OUTPUTS"));
    }

    #[test]
    fn g034_reducer_semantics_define_ordering_and_refuse_ambiguous_fanin() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("a", NodeKind::Const)
                    .output("out", "artifacts/a.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("b", NodeKind::Const)
                    .output("out", "artifacts/b.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("reduce", NodeKind::Const)
                    .semantic_kind(SemanticNodeKind::Reduce)
                    .input("in")
                    .output("result", "artifacts/reduce.json")
                    .build(),
            )
            .edge("a", "out", "reduce", "in")
            .edge("b", "out", "reduce", "in")
            .build();
        let report = validate_reducer_semantics(
            &graph,
            ReducerOrderingPolicyV1::LexicographicNodeId,
            false,
        );
        assert!(!report.valid);
        assert!(report
            .violations
            .iter()
            .any(|item| item.code == "R3403_REDUCER_AMBIGUOUS_FANIN"));
        assert_eq!(report.upstream_order[0].1, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn g035_conditional_edges_are_preserved_in_lowered_plan_semantics() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("branch", NodeKind::Const)
                    .semantic_kind(SemanticNodeKind::Branch)
                    .output("decision", "artifacts/decision.json")
                    .branch(BranchSpec {
                        decisions: vec!["left".to_string(), "right".to_string()],
                        default_decision: Some("right".to_string()),
                        decision_output: "decision".to_string(),
                    })
                    .build(),
            )
            .node(
                NodeBuilder::new("left_task", NodeKind::Const)
                    .trigger_rule(TriggerRule::AnySuccess)
                    .input("in")
                    .output("out", "artifacts/left.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("right_task", NodeKind::Const)
                    .trigger_rule(TriggerRule::AnySuccess)
                    .input("in")
                    .output("out", "artifacts/right.json")
                    .build(),
            )
            .edge("branch", "decision", "left_task", "in")
            .edge("branch", "decision", "right_task", "in")
            .build();
        let mut graph = graph;
        graph.edges[0].kind = EdgeKind::Conditional;
        graph.edges[0].decision = Some("left".to_string());
        graph.edges[1].kind = EdgeKind::Conditional;
        graph.edges[1].decision = Some("right".to_string());

        let snapshot = build_edge_semantics_snapshot(&graph).expect("snapshot");
        assert_eq!(
            snapshot
                .counts
                .iter()
                .find(|entry| entry.0 == "conditional")
                .map(|entry| entry.1)
                .unwrap_or_default(),
            2
        );
        assert!(snapshot
            .lowered_edges
            .iter()
            .any(|edge| edge.kind == "conditional" && edge.decision.as_deref() == Some("left")));
    }
}
