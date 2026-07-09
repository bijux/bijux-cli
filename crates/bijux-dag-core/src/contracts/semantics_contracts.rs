use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

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
    let success =
        parent_states.iter().filter(|state| **state == UpstreamTerminalStateV1::Success).count();
    let failed =
        parent_states.iter().filter(|state| **state == UpstreamTerminalStateV1::Failed).count();
    let cancelled =
        parent_states.iter().filter(|state| **state == UpstreamTerminalStateV1::Cancelled).count();
    let total = parent_states.len();

    let result = match trigger_rule {
        "all_success" => {
            (success == total, "requires every parent in success and treats skipped as non-success")
        }
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
                remediation: "connect barrier node to one or more upstream dependencies"
                    .to_string(),
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
    violations.sort_by(|left, right| {
        left.code.cmp(&right.code).then_with(|| left.node_id.cmp(&right.node_id))
    });
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
    violations.sort_by(|left, right| {
        left.code.cmp(&right.code).then_with(|| left.node_id.cmp(&right.node_id))
    });
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
        ("data".to_string(), graph.edges.iter().filter(|edge| edge.kind == EdgeKind::Data).count()),
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

/// Optional upstream presence evidence for one node input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalInputEvidenceV1 {
    /// Node identifier.
    pub node_id: String,
    /// Input port name.
    pub input: String,
    /// Whether this input is optional.
    pub optional: bool,
    /// Whether an upstream binding exists.
    pub bound: bool,
}

/// Optional upstream evidence report across graph nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalUpstreamEvidenceReportV1 {
    /// Input evidence records.
    pub records: Vec<OptionalInputEvidenceV1>,
    /// Missing optional inputs.
    pub missing_optional_inputs: Vec<String>,
    /// Missing required inputs.
    pub missing_required_inputs: Vec<String>,
}

/// Build optional-upstream evidence from graph plus optional-input declarations.
pub fn build_optional_upstream_evidence_report(
    graph: &Graph,
    optional_inputs: &BTreeMap<String, BTreeSet<String>>,
) -> OptionalUpstreamEvidenceReportV1 {
    let mut records = Vec::new();
    for node in &graph.nodes {
        let optional_set = optional_inputs.get(&node.id).cloned().unwrap_or_default();
        for input in &node.inputs {
            let bound =
                graph.edges.iter().any(|edge| edge.to.node_id == node.id && edge.to.port == *input);
            records.push(OptionalInputEvidenceV1 {
                node_id: node.id.clone(),
                input: input.clone(),
                optional: optional_set.contains(input),
                bound,
            });
        }
    }
    records.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then_with(|| left.input.cmp(&right.input))
            .then_with(|| left.optional.cmp(&right.optional))
    });

    let missing_optional_inputs = records
        .iter()
        .filter(|record| record.optional && !record.bound)
        .map(|record| format!("{}.{}", record.node_id, record.input))
        .collect::<Vec<_>>();
    let missing_required_inputs = records
        .iter()
        .filter(|record| !record.optional && !record.bound)
        .map(|record| format!("{}.{}", record.node_id, record.input))
        .collect::<Vec<_>>();

    OptionalUpstreamEvidenceReportV1 { records, missing_optional_inputs, missing_required_inputs }
}

/// Trigger rule profiles covered by semantic truth tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerRuleProfileV1 {
    AllSuccess,
    AllDone,
    AnySuccess,
    NoneFailed,
    QuorumSuccess,
    SkippedAwareAnySuccess,
}

/// Input row for trigger-rule truth table evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerTruthTableRowV1 {
    /// Trigger profile.
    pub profile: TriggerRuleProfileV1,
    /// Parent states in evaluation set.
    pub parent_states: Vec<UpstreamTerminalStateV1>,
    /// Optional quorum threshold for quorum profiles.
    pub quorum_threshold: Option<usize>,
    /// Whether row is runnable.
    pub runnable: bool,
}

/// Evaluate a single trigger-rule truth-table row.
pub fn evaluate_trigger_truth_table_row(
    profile: TriggerRuleProfileV1,
    parent_states: Vec<UpstreamTerminalStateV1>,
    quorum_threshold: Option<usize>,
) -> Result<TriggerTruthTableRowV1, String> {
    let success =
        parent_states.iter().filter(|state| **state == UpstreamTerminalStateV1::Success).count();
    let failed =
        parent_states.iter().filter(|state| **state == UpstreamTerminalStateV1::Failed).count();
    let skipped =
        parent_states.iter().filter(|state| **state == UpstreamTerminalStateV1::Skipped).count();
    let done = parent_states
        .iter()
        .filter(|state| {
            matches!(
                state,
                UpstreamTerminalStateV1::Success
                    | UpstreamTerminalStateV1::Failed
                    | UpstreamTerminalStateV1::Skipped
                    | UpstreamTerminalStateV1::Cancelled
            )
        })
        .count();
    let total = parent_states.len();

    let runnable = match profile {
        TriggerRuleProfileV1::AllSuccess => success == total,
        TriggerRuleProfileV1::AllDone => done == total,
        TriggerRuleProfileV1::AnySuccess => success > 0,
        TriggerRuleProfileV1::NoneFailed => failed == 0,
        TriggerRuleProfileV1::QuorumSuccess => {
            let threshold = quorum_threshold.ok_or("quorum threshold is required")?;
            success >= threshold
        }
        TriggerRuleProfileV1::SkippedAwareAnySuccess => {
            success > 0 || (success == 0 && skipped == total)
        }
    };

    Ok(TriggerTruthTableRowV1 { profile, parent_states, quorum_threshold, runnable })
}

/// Matrix expansion refusal details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixExpansionRefusalV1 {
    /// Stable refusal code.
    pub code: String,
    /// Reason details.
    pub reason: String,
}

/// Deterministic matrix expansion output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixExpansionReportV1 {
    /// Expanded instance identifiers.
    pub instances: Vec<String>,
    /// Expansion cardinality.
    pub cardinality: usize,
    /// Refusal when expansion is unsafe.
    pub refusal: Option<MatrixExpansionRefusalV1>,
}

/// Expand matrix dimensions with deterministic naming and hard cardinality cap.
pub fn expand_matrix_bounded(
    node_id: &str,
    dimensions: &BTreeMap<String, Vec<String>>,
    max_cardinality: usize,
) -> MatrixExpansionReportV1 {
    if dimensions.is_empty() {
        return MatrixExpansionReportV1 {
            instances: Vec::new(),
            cardinality: 0,
            refusal: Some(MatrixExpansionRefusalV1 {
                code: "M3801_EMPTY_DIMENSIONS".to_string(),
                reason: "matrix dimensions must be declared explicitly".to_string(),
            }),
        };
    }
    let mut cardinality = 1usize;
    for values in dimensions.values() {
        if values.is_empty() {
            return MatrixExpansionReportV1 {
                instances: Vec::new(),
                cardinality: 0,
                refusal: Some(MatrixExpansionRefusalV1 {
                    code: "M3802_UNBOUNDED_DIMENSION".to_string(),
                    reason: "dimension cannot be empty".to_string(),
                }),
            };
        }
        cardinality = cardinality.saturating_mul(values.len());
        if cardinality > max_cardinality {
            return MatrixExpansionReportV1 {
                instances: Vec::new(),
                cardinality,
                refusal: Some(MatrixExpansionRefusalV1 {
                    code: "M3803_EXPLOSIVE_CARDINALITY".to_string(),
                    reason: format!(
                        "matrix cardinality {cardinality} exceeds max {max_cardinality}"
                    ),
                }),
            };
        }
    }

    let mut keys = dimensions.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    let mut products = vec![BTreeMap::<String, String>::new()];
    for key in keys {
        let values = dimensions.get(&key).cloned().unwrap_or_default();
        let mut next = Vec::new();
        for partial in &products {
            for value in &values {
                let mut row = partial.clone();
                row.insert(key.clone(), value.clone());
                next.push(row);
            }
        }
        products = next;
    }
    let mut instances = products
        .into_iter()
        .map(|row| {
            let labels = row
                .into_iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(",");
            format!("{node_id}[{labels}]")
        })
        .collect::<Vec<_>>();
    instances.sort();

    MatrixExpansionReportV1 { instances, cardinality, refusal: None }
}

/// Partition identity with stable key and lineage reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionIdentityV1 {
    /// Dataset identifier.
    pub dataset_id: String,
    /// Stable partition key.
    pub partition_key: String,
    /// Stable lineage identifier.
    pub lineage_id: String,
    /// Reducer node that consumes this partition.
    pub reducer_node_id: String,
}

/// Stable partition identity report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionIdentityReportV1 {
    /// Sorted partition identities.
    pub partitions: Vec<PartitionIdentityV1>,
}

/// Build deterministic partition identities from partition dimensions.
pub fn build_partition_identity_report(
    dataset_id: &str,
    reducer_node_id: &str,
    partitions: Vec<BTreeMap<String, String>>,
) -> Result<PartitionIdentityReportV1, String> {
    if dataset_id.trim().is_empty() {
        return Err("dataset_id cannot be empty".to_string());
    }
    if reducer_node_id.trim().is_empty() {
        return Err("reducer_node_id cannot be empty".to_string());
    }
    let mut rows = partitions
        .into_iter()
        .map(|partition| {
            let key_material = partition
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("|");
            let partition_key = format!("{dataset_id}:{key_material}");
            let lineage_id = format!("{dataset_id}->{reducer_node_id}:{key_material}");
            PartitionIdentityV1 {
                dataset_id: dataset_id.to_string(),
                partition_key,
                lineage_id,
                reducer_node_id: reducer_node_id.to_string(),
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.partition_key.cmp(&right.partition_key));
    Ok(PartitionIdentityReportV1 { partitions: rows })
}

/// Subgraph node descriptor for scoped materialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphNodeDescriptorV1 {
    /// Local node identifier inside subgraph.
    pub local_id: String,
    /// Parameter names consumed by this node.
    pub params: Vec<String>,
    /// Artifact output names produced by this node.
    pub artifacts: Vec<String>,
}

/// Materialized subgraph node with scoped identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedSubgraphNodeV1 {
    /// Scoped node id.
    pub scoped_node_id: String,
    /// Scoped param keys.
    pub scoped_params: Vec<String>,
    /// Scoped artifact names.
    pub scoped_artifacts: Vec<String>,
    /// Replay ancestry chain.
    pub replay_ancestry: Vec<String>,
}

/// Subgraph materialization report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphMaterializationReportV1 {
    /// Whether scoped materialization succeeded.
    pub valid: bool,
    /// Materialized nodes.
    pub nodes: Vec<ScopedSubgraphNodeV1>,
    /// Collision errors.
    pub collisions: Vec<String>,
}

/// Materialize reusable subgraph nodes into scoped identities with ancestry.
pub fn materialize_subgraph_scope(
    parent_graph_id: &str,
    subgraph_id: &str,
    parent_run_id: &str,
    nodes: Vec<SubgraphNodeDescriptorV1>,
) -> SubgraphMaterializationReportV1 {
    let mut collisions = Vec::new();
    let mut seen_nodes = BTreeSet::new();
    let mut materialized = Vec::new();

    for node in nodes {
        let scoped_node_id = format!("{parent_graph_id}/{subgraph_id}/{}", node.local_id);
        if !seen_nodes.insert(scoped_node_id.clone()) {
            collisions.push(format!("duplicate scoped node id: {scoped_node_id}"));
            continue;
        }

        let mut scoped_params = node
            .params
            .iter()
            .map(|name| format!("{subgraph_id}.params.{name}"))
            .collect::<Vec<_>>();
        scoped_params.sort();
        scoped_params.dedup();
        if scoped_params.len() != node.params.len() {
            collisions.push(format!("duplicate param names on node {}", node.local_id));
        }

        let mut scoped_artifacts = node
            .artifacts
            .iter()
            .map(|name| format!("{subgraph_id}.artifacts.{name}"))
            .collect::<Vec<_>>();
        scoped_artifacts.sort();
        scoped_artifacts.dedup();
        if scoped_artifacts.len() != node.artifacts.len() {
            collisions.push(format!("duplicate artifact names on node {}", node.local_id));
        }

        materialized.push(ScopedSubgraphNodeV1 {
            scoped_node_id,
            scoped_params,
            scoped_artifacts,
            replay_ancestry: vec![parent_run_id.to_string(), subgraph_id.to_string()],
        });
    }

    materialized.sort_by(|left, right| left.scoped_node_id.cmp(&right.scoped_node_id));
    collisions.sort();
    SubgraphMaterializationReportV1 {
        valid: collisions.is_empty(),
        nodes: materialized,
        collisions,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_branch_decision_artifact, build_edge_semantics_snapshot,
        build_optional_upstream_evidence_report, build_partition_identity_report,
        evaluate_trigger_readiness_from_states, evaluate_trigger_truth_table_row,
        expand_matrix_bounded, materialize_subgraph_scope, validate_barrier_semantics,
        validate_reducer_semantics, ReducerOrderingPolicyV1, SubgraphNodeDescriptorV1,
        TriggerRuleProfileV1, UpstreamTerminalStateV1,
    };
    use crate::{
        BranchSpec, DagBuilder, EdgeKind, NodeBuilder, NodeKind, SemanticNodeKind, TriggerRule,
    };
    use std::collections::{BTreeMap, BTreeSet};

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
        assert!(report.violations.iter().any(|item| item.code == "B3301_BARRIER_REQUIRES_INPUTS"));
        assert!(report
            .violations
            .iter()
            .any(|item| item.code == "B3302_BARRIER_MUST_NOT_DECLARE_OUTPUTS"));
    }

    #[test]
    fn g034_reducer_semantics_define_ordering_and_refuse_ambiguous_fanin() {
        let graph = DagBuilder::new()
            .node(NodeBuilder::new("a", NodeKind::Const).output("out", "artifacts/a.json").build())
            .node(NodeBuilder::new("b", NodeKind::Const).output("out", "artifacts/b.json").build())
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
        let report =
            validate_reducer_semantics(&graph, ReducerOrderingPolicyV1::LexicographicNodeId, false);
        assert!(!report.valid);
        assert!(report.violations.iter().any(|item| item.code == "R3403_REDUCER_AMBIGUOUS_FANIN"));
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

    #[test]
    fn g036_optional_upstreams_are_reported_separately_from_required_missing_inputs() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("producer", NodeKind::Const)
                    .output("out", "artifacts/out.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("consumer", NodeKind::Const)
                    .input("required_in")
                    .input("optional_in")
                    .output("done", "artifacts/done.json")
                    .build(),
            )
            .edge("producer", "out", "consumer", "required_in")
            .build();
        let optional_inputs =
            BTreeMap::from([("consumer".to_string(), BTreeSet::from(["optional_in".to_string()]))]);

        let report = build_optional_upstream_evidence_report(&graph, &optional_inputs);
        assert_eq!(report.missing_required_inputs.len(), 0);
        assert_eq!(report.missing_optional_inputs, vec!["consumer.optional_in".to_string()]);
    }

    #[test]
    fn g037_trigger_rule_truth_tables_cover_quorum_and_skipped_aware_profiles() {
        let quorum_row = evaluate_trigger_truth_table_row(
            TriggerRuleProfileV1::QuorumSuccess,
            vec![
                UpstreamTerminalStateV1::Success,
                UpstreamTerminalStateV1::Skipped,
                UpstreamTerminalStateV1::Success,
            ],
            Some(2),
        )
        .expect("quorum row");
        assert!(quorum_row.runnable);

        let skipped_aware = evaluate_trigger_truth_table_row(
            TriggerRuleProfileV1::SkippedAwareAnySuccess,
            vec![UpstreamTerminalStateV1::Skipped, UpstreamTerminalStateV1::Skipped],
            None,
        )
        .expect("skipped-aware row");
        assert!(skipped_aware.runnable);
    }

    #[test]
    fn g038_matrix_expansion_is_bounded_and_refuses_explosive_cardinality() {
        let dimensions = BTreeMap::from([
            ("chromosome".to_string(), vec!["1".to_string(), "2".to_string()]),
            ("sample".to_string(), vec!["s1".to_string(), "s2".to_string()]),
        ]);
        let ok = expand_matrix_bounded("align", &dimensions, 8);
        assert!(ok.refusal.is_none());
        assert_eq!(ok.cardinality, 4);
        assert!(ok.instances[0].starts_with("align[chromosome="));

        let refused = expand_matrix_bounded("align", &dimensions, 3);
        assert_eq!(
            refused.refusal.as_ref().map(|value| value.code.as_str()),
            Some("M3803_EXPLOSIVE_CARDINALITY")
        );
    }

    #[test]
    fn g039_partition_identities_are_stable_and_reducer_linked() {
        let report = build_partition_identity_report(
            "dataset.v1",
            "reduce.partitions",
            vec![
                BTreeMap::from([
                    ("chromosome".to_string(), "2".to_string()),
                    ("sample".to_string(), "s2".to_string()),
                ]),
                BTreeMap::from([
                    ("chromosome".to_string(), "1".to_string()),
                    ("sample".to_string(), "s1".to_string()),
                ]),
            ],
        )
        .expect("partition report");
        assert_eq!(report.partitions.len(), 2);
        assert!(report.partitions[0].partition_key < report.partitions[1].partition_key);
        assert!(report.partitions.iter().all(|row| row.lineage_id.contains("reduce.partitions")));
    }

    #[test]
    fn g040_subgraph_materialization_scopes_ids_and_catches_collisions() {
        let report = materialize_subgraph_scope(
            "graph.main",
            "subgraph.align",
            "run.42",
            vec![
                SubgraphNodeDescriptorV1 {
                    local_id: "step".to_string(),
                    params: vec!["threads".to_string()],
                    artifacts: vec!["bam".to_string()],
                },
                SubgraphNodeDescriptorV1 {
                    local_id: "step".to_string(),
                    params: vec!["threads".to_string()],
                    artifacts: vec!["bam".to_string()],
                },
            ],
        );
        assert!(!report.valid);
        assert!(report.collisions.iter().any(|entry| entry.contains("duplicate scoped node id")));
        assert!(report
            .nodes
            .iter()
            .all(|node| node.replay_ancestry
                == vec!["run.42".to_string(), "subgraph.align".to_string()]));
    }
}
