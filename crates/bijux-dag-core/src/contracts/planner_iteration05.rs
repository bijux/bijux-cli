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

/// Semantic plan diff report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticPlanDiffReportV1 {
    /// Whether execution semantics changed.
    pub semantics_changed: bool,
    /// Topology changed flag.
    pub topology_changed: bool,
    /// Parameter surface changed flag.
    pub params_changed: bool,
    /// Resource hint changed flag.
    pub resources_changed: bool,
    /// Changed node ids.
    pub changed_nodes: Vec<String>,
}

/// Compare plans by execution semantics and ignore formatting/metadata noise.
pub fn diff_plans_semantically(
    before: &Graph,
    after: &Graph,
) -> Result<SemanticPlanDiffReportV1, GraphError> {
    let before_plan = lower_graph_to_execution_plan(&compile_graph(before)?.normalized_graph, Default::default())
        .map_err(|_| GraphError::ValidationFailed)?;
    let after_plan = lower_graph_to_execution_plan(&compile_graph(after)?.normalized_graph, Default::default())
        .map_err(|_| GraphError::ValidationFailed)?;

    let semantics_changed = before_plan.execution_fingerprint != after_plan.execution_fingerprint;
    let topology_changed = before_plan.ordering != after_plan.ordering || before_plan.edges != after_plan.edges;

    let before_nodes = before_plan
        .nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                (
                    node.io_contract.param_bindings.clone(),
                    node.resources.as_ref().map(|value| (value.cpu, value.mem_mb)),
                ),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let after_nodes = after_plan
        .nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                (
                    node.io_contract.param_bindings.clone(),
                    node.resources.as_ref().map(|value| (value.cpu, value.mem_mb)),
                ),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut changed_nodes = before_nodes
        .keys()
        .chain(after_nodes.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter_map(|node_id| {
            if before_nodes.get(node_id) != after_nodes.get(node_id) {
                Some((*node_id).clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    changed_nodes.sort();
    let params_changed = changed_nodes.iter().any(|node_id| {
        before_nodes
            .get(node_id)
            .zip(after_nodes.get(node_id))
            .map(|(before_node, after_node)| before_node.0 != after_node.0)
            .unwrap_or(true)
    });
    let resources_changed = changed_nodes.iter().any(|node_id| {
        before_nodes
            .get(node_id)
            .zip(after_nodes.get(node_id))
            .map(|(before_node, after_node)| before_node.1 != after_node.1)
            .unwrap_or(true)
    });

    Ok(SemanticPlanDiffReportV1 {
        semantics_changed,
        topology_changed,
        params_changed,
        resources_changed,
        changed_nodes,
    })
}

/// Parameter source kind in effective resolution chains.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterSourceKindV1 {
    Default,
    Graph,
    Config,
    Env,
    CliOverride,
    AdapterConstraint,
}

/// One source event in parameter resolution chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterSourceEventV1 {
    /// Source kind.
    pub source: ParameterSourceKindV1,
    /// Source key/path.
    pub key: String,
    /// Source value representation.
    pub value: String,
}

/// Effective parameter resolution item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectiveParameterResolutionV1 {
    /// Effective parameter key.
    pub key: String,
    /// Effective parameter value.
    pub effective_value: String,
    /// Ordered source chain from low to high precedence.
    pub sources: Vec<ParameterSourceEventV1>,
}

/// Parameter explain report for operator-facing visibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterExplainReportV1 {
    /// Effective parameter entries.
    pub parameters: Vec<EffectiveParameterResolutionV1>,
}

/// Resolve parameters with source-chain visibility across all planning surfaces.
pub fn build_parameter_explain_report(
    defaults: &std::collections::BTreeMap<String, String>,
    graph_params: &std::collections::BTreeMap<String, String>,
    config_params: &std::collections::BTreeMap<String, String>,
    env_params: &std::collections::BTreeMap<String, String>,
    cli_overrides: &std::collections::BTreeMap<String, String>,
    adapter_constraints: &std::collections::BTreeMap<String, String>,
) -> ParameterExplainReportV1 {
    let keys = defaults
        .keys()
        .chain(graph_params.keys())
        .chain(config_params.keys())
        .chain(env_params.keys())
        .chain(cli_overrides.keys())
        .chain(adapter_constraints.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut parameters = keys
        .into_iter()
        .map(|key| {
            let mut chain = Vec::new();
            if let Some(value) = defaults.get(&key) {
                chain.push(ParameterSourceEventV1 {
                    source: ParameterSourceKindV1::Default,
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            if let Some(value) = graph_params.get(&key) {
                chain.push(ParameterSourceEventV1 {
                    source: ParameterSourceKindV1::Graph,
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            if let Some(value) = config_params.get(&key) {
                chain.push(ParameterSourceEventV1 {
                    source: ParameterSourceKindV1::Config,
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            if let Some(value) = env_params.get(&key) {
                chain.push(ParameterSourceEventV1 {
                    source: ParameterSourceKindV1::Env,
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            if let Some(value) = cli_overrides.get(&key) {
                chain.push(ParameterSourceEventV1 {
                    source: ParameterSourceKindV1::CliOverride,
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            if let Some(value) = adapter_constraints.get(&key) {
                chain.push(ParameterSourceEventV1 {
                    source: ParameterSourceKindV1::AdapterConstraint,
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            let effective_value = chain
                .last()
                .map(|event| event.value.clone())
                .unwrap_or_else(|| "null".to_string());
            EffectiveParameterResolutionV1 { key, effective_value, sources: chain }
        })
        .collect::<Vec<_>>();
    parameters.sort_by(|left, right| left.key.cmp(&right.key));
    ParameterExplainReportV1 { parameters }
}

/// Availability map for preflight capability checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanPreflightCapabilitiesV1 {
    pub shell_available: bool,
    pub container_available: bool,
    pub network_allowed: bool,
    pub filesystem_writable: bool,
    pub max_cpu: u32,
    pub max_mem_mb: u32,
}

/// Preflight diagnostic row for one node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanPreflightDiagnosticV1 {
    pub node_id: String,
    pub code: String,
    pub message: String,
}

/// Preflight report integrated into planning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanPreflightReportV1 {
    pub runnable: bool,
    pub diagnostics: Vec<PlanPreflightDiagnosticV1>,
}

/// Evaluate planning preflight checks for adapter/runtime capability availability.
pub fn run_plan_preflight(
    graph: &Graph,
    capabilities: &PlanPreflightCapabilitiesV1,
) -> PlanPreflightReportV1 {
    let mut diagnostics = Vec::new();
    for node in &graph.nodes {
        match node.kind {
            crate::NodeKind::Shell if !capabilities.shell_available => {
                diagnostics.push(PlanPreflightDiagnosticV1 {
                    node_id: node.id.clone(),
                    code: "PF4501_SHELL_UNAVAILABLE".to_string(),
                    message: "shell adapter unavailable in planner preflight".to_string(),
                });
            }
            crate::NodeKind::Container if !capabilities.container_available => {
                diagnostics.push(PlanPreflightDiagnosticV1 {
                    node_id: node.id.clone(),
                    code: "PF4502_CONTAINER_UNAVAILABLE".to_string(),
                    message: "container adapter unavailable in planner preflight".to_string(),
                });
            }
            _ => {}
        }
        if node.effects.contains(&crate::Effect::Network) && !capabilities.network_allowed {
            diagnostics.push(PlanPreflightDiagnosticV1 {
                node_id: node.id.clone(),
                code: "PF4503_NETWORK_BLOCKED".to_string(),
                message: "network side effects are disallowed in this planning profile".to_string(),
            });
        }
        if node.effects.contains(&crate::Effect::Filesystem) && !capabilities.filesystem_writable {
            diagnostics.push(PlanPreflightDiagnosticV1 {
                node_id: node.id.clone(),
                code: "PF4504_FILESYSTEM_BLOCKED".to_string(),
                message: "filesystem writes unavailable for planner preflight".to_string(),
            });
        }
        if let Some(resources) = &node.resources {
            if resources.cpu > capabilities.max_cpu {
                diagnostics.push(PlanPreflightDiagnosticV1 {
                    node_id: node.id.clone(),
                    code: "PF4505_CPU_EXCEEDS_CAP".to_string(),
                    message: format!("cpu hint {} exceeds max {}", resources.cpu, capabilities.max_cpu),
                });
            }
            if resources.mem_mb > capabilities.max_mem_mb {
                diagnostics.push(PlanPreflightDiagnosticV1 {
                    node_id: node.id.clone(),
                    code: "PF4506_MEM_EXCEEDS_CAP".to_string(),
                    message: format!(
                        "memory hint {} exceeds max {}",
                        resources.mem_mb, capabilities.max_mem_mb
                    ),
                });
            }
        }
    }
    diagnostics.sort_by(|left, right| left.node_id.cmp(&right.node_id).then_with(|| left.code.cmp(&right.code)));
    PlanPreflightReportV1 { runnable: diagnostics.is_empty(), diagnostics }
}

/// Plan fingerprint trust/explain report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanFingerprintTrustReportV1 {
    /// Execution fingerprint before mutation.
    pub before_execution_fingerprint: String,
    /// Execution fingerprint after mutation.
    pub after_execution_fingerprint: String,
    /// Whether execution fingerprint changed.
    pub fingerprint_changed: bool,
    /// Explain factors for mismatch.
    pub mismatch_factors: Vec<String>,
}

/// Compare plan fingerprints and report execution-relevant mismatch factors only.
pub fn explain_plan_fingerprint_trust(
    before: &Graph,
    after: &Graph,
) -> Result<PlanFingerprintTrustReportV1, GraphError> {
    let before_plan = lower_graph_to_execution_plan(&compile_graph(before)?.normalized_graph, Default::default())
        .map_err(|_| GraphError::ValidationFailed)?;
    let after_plan = lower_graph_to_execution_plan(&compile_graph(after)?.normalized_graph, Default::default())
        .map_err(|_| GraphError::ValidationFailed)?;
    let mut mismatch_factors = Vec::new();

    if before_plan.ordering != after_plan.ordering || before_plan.edges != after_plan.edges {
        mismatch_factors.push("topology".to_string());
    }
    let before_node_map = before_plan
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<std::collections::BTreeMap<_, _>>();
    let after_node_map = after_plan
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<std::collections::BTreeMap<_, _>>();
    for node_id in before_node_map
        .keys()
        .chain(after_node_map.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
    {
        match (before_node_map.get(&node_id), after_node_map.get(&node_id)) {
            (Some(before_node), Some(after_node)) => {
                if before_node.io_contract.param_bindings != after_node.io_contract.param_bindings {
                    mismatch_factors.push(format!("params:{node_id}"));
                }
                if before_node.retry.max_attempts != after_node.retry.max_attempts
                    || before_node.retry.backoff_ms != after_node.retry.backoff_ms
                {
                    mismatch_factors.push(format!("retry:{node_id}"));
                }
                if before_node.side_effects != after_node.side_effects {
                    mismatch_factors.push(format!("effects:{node_id}"));
                }
                if before_node.outputs != after_node.outputs {
                    mismatch_factors.push(format!("artifacts:{node_id}"));
                }
            }
            _ => mismatch_factors.push(format!("node_set:{node_id}")),
        }
    }
    mismatch_factors.sort();
    mismatch_factors.dedup();

    Ok(PlanFingerprintTrustReportV1 {
        before_execution_fingerprint: before_plan.execution_fingerprint.clone(),
        after_execution_fingerprint: after_plan.execution_fingerprint.clone(),
        fingerprint_changed: before_plan.execution_fingerprint != after_plan.execution_fingerprint,
        mismatch_factors,
    })
}

/// Portable plan package for offline review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanPackageExportV1 {
    /// Package format version.
    pub package_version: String,
    /// Canonical graph JSON.
    pub graph_canonical_json: String,
    /// Plan JSON.
    pub plan_json: String,
    /// Referenced schema versions.
    pub schema_refs: Vec<String>,
    /// Planned artifact expectations.
    pub expected_artifacts: Vec<String>,
    /// Capability decisions captured by planner.
    pub capability_decisions: Vec<String>,
}

/// Export portable plan package with review-ready plan evidence.
pub fn export_plan_package(
    graph: &Graph,
    schema_refs: Vec<String>,
    capability_decisions: Vec<String>,
) -> Result<PlanPackageExportV1, GraphError> {
    let compile = compile_graph(graph)?;
    let plan = lower_graph_to_execution_plan(&compile.normalized_graph, Default::default())
        .map_err(|_| GraphError::ValidationFailed)?;
    let graph_canonical_json = compile.normalized_graph.to_canonical_json()?;
    let plan_json = serde_json::to_string_pretty(&plan)?;
    let mut expected_artifacts = plan
        .nodes
        .iter()
        .flat_map(|node| {
            node.outputs
                .iter()
                .map(move |output| format!("{}:{}", node.id, output.path))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    expected_artifacts.sort();
    expected_artifacts.dedup();

    Ok(PlanPackageExportV1 {
        package_version: "bijux-plan-package/v1".to_string(),
        graph_canonical_json,
        plan_json,
        schema_refs,
        expected_artifacts,
        capability_decisions,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_complete_dry_plan_output, build_parameter_explain_report, build_plan_explain_report,
        diff_plans_semantically, explain_plan_fingerprint_trust, export_plan_package,
        run_plan_preflight, ParameterSourceKindV1, PlanPreflightCapabilitiesV1,
    };
    use crate::{DagBuilder, Effect, GraphMeta, NodeBuilder, NodeKind, Resources};
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

    #[test]
    fn g043_semantic_plan_diff_ignores_metadata_noise() {
        let base = DagBuilder::new()
            .node(
                NodeBuilder::new("n1", NodeKind::Const)
                    .output("out", "artifacts/n1.json")
                    .build(),
            )
            .build();
        let with_meta = DagBuilder::new()
            .graph_meta(GraphMeta {
                name: "renamed".to_string(),
                description: Some("metadata-only change".to_string()),
                owners: vec![],
                tags: vec!["doc".to_string()],
            })
            .node(
                NodeBuilder::new("n1", NodeKind::Const)
                    .output("out", "artifacts/n1.json")
                    .build(),
            )
            .build();

        let diff = diff_plans_semantically(&base, &with_meta).expect("semantic diff");
        assert!(!diff.semantics_changed);
        assert!(diff.changed_nodes.is_empty());
    }

    #[test]
    fn g044_parameter_resolution_report_exposes_full_source_chain() {
        let report = build_parameter_explain_report(
            &std::collections::BTreeMap::from([("threads".to_string(), "4".to_string())]),
            &std::collections::BTreeMap::from([("threads".to_string(), "8".to_string())]),
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::from([("threads".to_string(), "12".to_string())]),
            &std::collections::BTreeMap::from([("threads".to_string(), "16".to_string())]),
            &std::collections::BTreeMap::from([("threads".to_string(), "max=14".to_string())]),
        );
        let threads = report
            .parameters
            .iter()
            .find(|entry| entry.key == "threads")
            .expect("threads parameter should resolve");
        assert_eq!(threads.effective_value, "max=14");
        assert_eq!(
            threads.sources.last().map(|entry| entry.source.clone()),
            Some(ParameterSourceKindV1::AdapterConstraint)
        );
    }

    #[test]
    fn g045_planner_preflight_converts_missing_capabilities_into_diagnostics() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("shell_step", NodeKind::Shell)
                    .output("out", "artifacts/shell.json")
                    .effect(Effect::Filesystem)
                    .effect(Effect::Network)
                    .build(),
            )
            .node(
                NodeBuilder::new("heavy_step", NodeKind::Const)
                    .output("out", "artifacts/heavy.json")
                    .build(),
            )
            .build();
        let mut graph = graph;
        graph.nodes[1].resources = Some(Resources { cpu: 64, mem_mb: 131_072 });
        let capabilities = PlanPreflightCapabilitiesV1 {
            shell_available: false,
            container_available: true,
            network_allowed: false,
            filesystem_writable: false,
            max_cpu: 32,
            max_mem_mb: 65536,
        };
        let report = run_plan_preflight(&graph, &capabilities);
        assert!(!report.runnable);
        assert!(report
            .diagnostics
            .iter()
            .any(|diag| diag.code == "PF4501_SHELL_UNAVAILABLE"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diag| diag.code == "PF4505_CPU_EXCEEDS_CAP"));
    }

    #[test]
    fn g046_plan_fingerprint_tracks_execution_semantics_not_metadata_noise() {
        let base = DagBuilder::new()
            .node(
                NodeBuilder::new("n1", NodeKind::Const)
                    .output("out", "artifacts/n1.json")
                    .build(),
            )
            .build();
        let metadata_only = DagBuilder::new()
            .graph_meta(GraphMeta {
                name: "renamed".to_string(),
                description: Some("noise".to_string()),
                owners: vec![],
                tags: vec!["meta".to_string()],
            })
            .node(
                NodeBuilder::new("n1", NodeKind::Const)
                    .output("out", "artifacts/n1.json")
                    .build(),
            )
            .build();
        let trust = explain_plan_fingerprint_trust(&base, &metadata_only).expect("trust");
        assert!(!trust.fingerprint_changed);

        let semantic_change = DagBuilder::new()
            .node(
                NodeBuilder::new("n1", NodeKind::Const)
                    .output("out", "artifacts/renamed.json")
                    .build(),
            )
            .build();
        let changed = explain_plan_fingerprint_trust(&base, &semantic_change).expect("changed");
        assert!(changed.fingerprint_changed);
        assert!(changed
            .mismatch_factors
            .iter()
            .any(|factor| factor.starts_with("artifacts:")));
    }

    #[test]
    fn g047_plan_package_export_includes_portable_review_surfaces() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("n1", NodeKind::Const)
                    .output("out", "artifacts/n1.json")
                    .build(),
            )
            .build();
        let package = export_plan_package(
            &graph,
            vec!["bijux-dag/v0.1".to_string()],
            vec!["executor:const=available".to_string()],
        )
        .expect("plan package");
        assert_eq!(package.package_version, "bijux-plan-package/v1");
        assert!(package.graph_canonical_json.contains("\"nodes\""));
        assert!(package.plan_json.contains("\"nodes\""));
        assert!(package.expected_artifacts.iter().any(|entry| entry.contains("artifacts/n1.json")));
    }
}
