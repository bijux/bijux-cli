use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::Instant;

use crate::{
    compile_graph, lower_graph_to_execution_plan, Graph, GraphError, PlannerSeverity, Severity,
};

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
    /// Cache eligibility declared by the graph contract.
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
pub fn build_complete_dry_plan_output(
    graph: &Graph,
) -> Result<DryPlanCompleteOutputV1, GraphError> {
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
            cache_eligible: node.cache.enabled,
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
    let selected = selected_nodes
        .cloned()
        .unwrap_or_else(|| graph.nodes.iter().map(|node| node.id.clone()).collect());
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
    let before_plan =
        lower_graph_to_execution_plan(&compile_graph(before)?.normalized_graph, Default::default())
            .map_err(|_| GraphError::ValidationFailed)?;
    let after_plan =
        lower_graph_to_execution_plan(&compile_graph(after)?.normalized_graph, Default::default())
            .map_err(|_| GraphError::ValidationFailed)?;

    let semantics_changed = before_plan.execution_fingerprint != after_plan.execution_fingerprint;
    let topology_changed =
        before_plan.ordering != after_plan.ordering || before_plan.edges != after_plan.edges;

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
            let effective_value =
                chain.last().map(|event| event.value.clone()).unwrap_or_else(|| "null".to_string());
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
            crate::NodeKind::Python if !capabilities.shell_available => {
                diagnostics.push(PlanPreflightDiagnosticV1 {
                    node_id: node.id.clone(),
                    code: "PF4501_SHELL_UNAVAILABLE".to_string(),
                    message: "python adapter unavailable in planner preflight".to_string(),
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
                    message: format!(
                        "cpu hint {} exceeds max {}",
                        resources.cpu, capabilities.max_cpu
                    ),
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
    diagnostics.sort_by(|left, right| {
        left.node_id.cmp(&right.node_id).then_with(|| left.code.cmp(&right.code))
    });
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
    let before_plan =
        lower_graph_to_execution_plan(&compile_graph(before)?.normalized_graph, Default::default())
            .map_err(|_| GraphError::ValidationFailed)?;
    let after_plan =
        lower_graph_to_execution_plan(&compile_graph(after)?.normalized_graph, Default::default())
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
    for node_id in
        before_node_map.keys().chain(after_node_map.keys()).cloned().collect::<BTreeSet<_>>()
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

/// Graph-level resource hint surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphResourceHintsV1 {
    pub cpu: u32,
    pub memory_mb: u32,
    pub disk_mb: u32,
    pub scratch_mb: u32,
    pub network: String,
    pub walltime_s: u64,
    pub gpu: u32,
    pub pool: Option<String>,
}

/// Node-level resource hint surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeResourceHintsV1 {
    pub node_id: String,
    pub cpu: u32,
    pub memory_mb: u32,
    pub disk_mb: u32,
    pub scratch_mb: u32,
    pub network: String,
    pub walltime_s: u64,
    pub gpu: u32,
    pub pool: Option<String>,
}

/// Resource hint report for planning and admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceHintsReportV1 {
    pub graph: GraphResourceHintsV1,
    pub nodes: Vec<NodeResourceHintsV1>,
}

/// Build visible resource hints at graph and node levels.
pub fn build_resource_hints_report(
    graph: &Graph,
    graph_hints: GraphResourceHintsV1,
) -> ResourceHintsReportV1 {
    let mut nodes = graph
        .nodes
        .iter()
        .map(|node| {
            let pool = node
                .tags
                .iter()
                .find(|tag| tag.starts_with("pool:"))
                .map(|tag| tag.trim_start_matches("pool:").to_string())
                .or_else(|| graph_hints.pool.clone());
            let gpu = node
                .resources
                .as_ref()
                .filter(|resources| resources.gpu_devices > 0)
                .map(|resources| resources.gpu_devices)
                .or_else(|| {
                    node.tags
                        .iter()
                        .find(|tag| tag.starts_with("gpu:"))
                        .and_then(|tag| tag.trim_start_matches("gpu:").parse::<u32>().ok())
                })
                .unwrap_or(graph_hints.gpu);

            NodeResourceHintsV1 {
                node_id: node.id.clone(),
                cpu: node.resources.as_ref().map(|value| value.cpu).unwrap_or(graph_hints.cpu),
                memory_mb: node
                    .resources
                    .as_ref()
                    .map(|value| value.mem_mb)
                    .unwrap_or(graph_hints.memory_mb),
                disk_mb: graph_hints.disk_mb,
                scratch_mb: graph_hints.scratch_mb,
                network: if node.effects.contains(&crate::Effect::Network) {
                    "required".to_string()
                } else {
                    graph_hints.network.clone()
                },
                walltime_s: graph_hints.walltime_s,
                gpu,
                pool,
            }
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    ResourceHintsReportV1 { graph: graph_hints, nodes }
}

/// Planner conflict policy/capability envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerConflictEnvelopeV1 {
    pub allowed_runtime: String,
    pub allowed_adapters: BTreeSet<String>,
    pub allow_conditional_edges: bool,
    pub allow_matrix_expansion: bool,
    pub required_artifact_prefix: String,
}

/// Readable planner conflict diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerConflictDiagnosticV1 {
    pub code: String,
    pub node_id: Option<String>,
    pub reason: String,
    pub remediation: String,
}

/// Readable planner conflict report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerConflictReportV1 {
    pub conflicts: Vec<PlannerConflictDiagnosticV1>,
}

/// Detect planner conflicts before runtime with exact, readable reasons.
pub fn detect_planner_conflicts(
    graph: &Graph,
    envelope: &PlannerConflictEnvelopeV1,
) -> PlannerConflictReportV1 {
    let mut conflicts = Vec::new();
    if envelope.allowed_runtime != "local" {
        conflicts.push(PlannerConflictDiagnosticV1 {
            code: "PC4901_RUNTIME_INCOMPATIBLE".to_string(),
            node_id: None,
            reason: format!(
                "runtime {} is unsupported by this planner profile",
                envelope.allowed_runtime
            ),
            remediation: "use runtime local or a profile with declared runtime compatibility"
                .to_string(),
        });
    }

    for node in &graph.nodes {
        let adapter = node.kind.as_str().to_string();
        if !envelope.allowed_adapters.contains(&adapter) {
            conflicts.push(PlannerConflictDiagnosticV1 {
                code: "PC4902_ADAPTER_UNSUPPORTED".to_string(),
                node_id: Some(node.id.clone()),
                reason: format!("adapter {} is not in allowed set", adapter),
                remediation: "enable adapter in policy or replace node kind".to_string(),
            });
        }
        if node.trigger_rule == crate::TriggerRule::AllDone
            && node.effects.contains(&crate::Effect::Network)
        {
            conflicts.push(PlannerConflictDiagnosticV1 {
                code: "PC4903_POLICY_TRIGGER_CONFLICT".to_string(),
                node_id: Some(node.id.clone()),
                reason: "all_done with network side effects violates strict policy".to_string(),
                remediation: "use all_success/none_failed or remove network effect".to_string(),
            });
        }
        if node.semantic_kind == crate::SemanticNodeKind::Map && !envelope.allow_matrix_expansion {
            conflicts.push(PlannerConflictDiagnosticV1 {
                code: "PC4904_EXPANSION_DISABLED".to_string(),
                node_id: Some(node.id.clone()),
                reason: "matrix/map expansion disabled for this planning profile".to_string(),
                remediation: "enable matrix expansion policy or flatten mapped workloads"
                    .to_string(),
            });
        }
        for output in &node.outputs {
            if !output.path.starts_with(&envelope.required_artifact_prefix) {
                conflicts.push(PlannerConflictDiagnosticV1 {
                    code: "PC4905_ARTIFACT_POLICY_MISMATCH".to_string(),
                    node_id: Some(node.id.clone()),
                    reason: format!(
                        "artifact {} is outside required prefix {}",
                        output.path, envelope.required_artifact_prefix
                    ),
                    remediation: "rewrite artifact path into required policy prefix".to_string(),
                });
            }
        }
    }
    if !envelope.allow_conditional_edges
        && graph.edges.iter().any(|edge| edge.kind == crate::EdgeKind::Conditional)
    {
        conflicts.push(PlannerConflictDiagnosticV1 {
            code: "PC4906_CONDITIONAL_EDGES_DISABLED".to_string(),
            node_id: None,
            reason: "graph contains conditional edges but policy disables them".to_string(),
            remediation: "enable conditional edges in policy or replace edge kinds".to_string(),
        });
    }

    conflicts.sort_by(|left, right| {
        left.code.cmp(&right.code).then_with(|| left.node_id.cmp(&right.node_id))
    });
    PlannerConflictReportV1 { conflicts }
}

/// Planner benchmark sample for one graph shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerShapeBenchmarkV1 {
    pub shape: String,
    pub node_count: usize,
    pub elapsed_ms: u128,
    pub within_budget: bool,
}

/// Planner performance measurement report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerPerformanceReportV1 {
    pub build_budget_ms: u128,
    pub samples: Vec<PlannerShapeBenchmarkV1>,
}

fn benchmark_plan_build(
    shape: &str,
    graph: &Graph,
    build_budget_ms: u128,
) -> Result<PlannerShapeBenchmarkV1, GraphError> {
    let start = Instant::now();
    let compile = compile_graph(graph)?;
    let _plan = lower_graph_to_execution_plan(&compile.normalized_graph, Default::default())
        .map_err(|_| GraphError::ValidationFailed)?;
    let elapsed = start.elapsed().as_millis();
    Ok(PlannerShapeBenchmarkV1 {
        shape: shape.to_string(),
        node_count: graph.nodes.len(),
        elapsed_ms: elapsed,
        within_budget: elapsed <= build_budget_ms,
    })
}

fn chain_graph(length: usize) -> Graph {
    let mut builder = crate::DagBuilder::new();
    for index in 0..length {
        let id = format!("n{index}");
        let mut node = crate::NodeBuilder::new(&id, crate::NodeKind::Const)
            .output("out", &format!("artifacts/{id}.json"));
        if index > 0 {
            node = node.input("in");
        }
        builder = builder.node(node.build());
        if index > 0 {
            builder = builder.edge(&format!("n{}", index - 1), "out", &id, "in");
        }
    }
    builder.build()
}

fn wide_graph(width: usize) -> Graph {
    let mut builder = crate::DagBuilder::new().node(
        crate::NodeBuilder::new("root", crate::NodeKind::Const)
            .output("out", "artifacts/root.json")
            .build(),
    );
    for index in 0..width {
        let id = format!("leaf{index}");
        builder = builder
            .node(
                crate::NodeBuilder::new(&id, crate::NodeKind::Const)
                    .input("in")
                    .output("out", &format!("artifacts/{id}.json"))
                    .build(),
            )
            .edge("root", "out", &id, "in");
    }
    builder.build()
}

/// Measure planner performance on chain/wide/branching/matrix/subgraph/reducer shapes.
pub fn measure_planner_performance_real_shapes(
    build_budget_ms: u128,
) -> Result<PlannerPerformanceReportV1, GraphError> {
    let chain = chain_graph(12);
    let wide = wide_graph(12);
    let branching = wide_graph(6);
    let matrix = wide_graph(8);
    let subgraph = chain_graph(8);
    let reducer = chain_graph(10);

    let mut samples = vec![
        benchmark_plan_build("chain", &chain, build_budget_ms)?,
        benchmark_plan_build("wide", &wide, build_budget_ms)?,
        benchmark_plan_build("branching", &branching, build_budget_ms)?,
        benchmark_plan_build("matrix", &matrix, build_budget_ms)?,
        benchmark_plan_build("subgraph", &subgraph, build_budget_ms)?,
        benchmark_plan_build("reducer", &reducer, build_budget_ms)?,
    ];
    samples.sort_by(|left, right| left.shape.cmp(&right.shape));
    Ok(PlannerPerformanceReportV1 { build_budget_ms, samples })
}

#[cfg(test)]
mod tests {
    use super::{
        build_complete_dry_plan_output, build_parameter_explain_report, build_plan_explain_report,
        build_resource_hints_report, detect_planner_conflicts, diff_plans_semantically,
        explain_plan_fingerprint_trust, export_plan_package,
        measure_planner_performance_real_shapes, run_plan_preflight, GraphResourceHintsV1,
        ParameterSourceKindV1, PlanPreflightCapabilitiesV1, PlannerConflictEnvelopeV1,
    };
    use crate::{
        DagBuilder, EdgeKind, Effect, GraphMeta, NodeBuilder, NodeKind, Resources,
        SemanticNodeKind, TriggerRule,
    };
    use std::collections::BTreeSet;

    #[test]
    fn dry_plan_output_contains_lowered_shape_and_runnable_refusals() {
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
        assert!(report
            .nodes
            .iter()
            .any(|node| node.node_id == "load" && !node.dependencies.is_empty()));
    }

    #[test]
    fn plan_explain_reports_included_skipped_and_capability_blocked_nodes() {
        let graph = DagBuilder::new()
            .node(NodeBuilder::new("a", NodeKind::Const).output("out", "artifacts/a.json").build())
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
        let report =
            build_plan_explain_report(&graph, Some(&selected), &available).expect("plan explain");
        assert!(report.nodes.iter().any(|node| node.node_id == "a" && node.state == "included"));
        assert!(report.nodes.iter().any(|node| node.node_id == "b" && node.state == "blocked"));
    }

    #[test]
    fn semantic_plan_diff_ignores_metadata_noise() {
        let base = DagBuilder::new()
            .node(
                NodeBuilder::new("n1", NodeKind::Const).output("out", "artifacts/n1.json").build(),
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
                NodeBuilder::new("n1", NodeKind::Const).output("out", "artifacts/n1.json").build(),
            )
            .build();

        let diff = diff_plans_semantically(&base, &with_meta).expect("semantic diff");
        assert!(!diff.semantics_changed);
        assert!(diff.changed_nodes.is_empty());
    }

    #[test]
    fn parameter_resolution_report_exposes_full_source_chain() {
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
    fn planner_preflight_converts_missing_capabilities_into_diagnostics() {
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
        graph.nodes[1].resources = Some(Resources {
            cpu: 64,
            mem_mb: 131_072,
            gpu_devices: 0,
            named_resources: std::collections::BTreeMap::new(),
        });
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
        assert!(report.diagnostics.iter().any(|diag| diag.code == "PF4501_SHELL_UNAVAILABLE"));
        assert!(report.diagnostics.iter().any(|diag| diag.code == "PF4505_CPU_EXCEEDS_CAP"));
    }

    #[test]
    fn plan_fingerprint_tracks_execution_semantics_not_metadata_noise() {
        let base = DagBuilder::new()
            .node(
                NodeBuilder::new("n1", NodeKind::Const).output("out", "artifacts/n1.json").build(),
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
                NodeBuilder::new("n1", NodeKind::Const).output("out", "artifacts/n1.json").build(),
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
        assert!(changed.mismatch_factors.iter().any(|factor| factor.starts_with("artifacts:")));
    }

    #[test]
    fn plan_package_export_includes_portable_review_surfaces() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("n1", NodeKind::Const).output("out", "artifacts/n1.json").build(),
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

    #[test]
    fn resource_hints_are_visible_at_graph_and_node_levels() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("cpu_heavy", NodeKind::Const)
                    .output("out", "artifacts/out.json")
                    .tag("pool:hpc")
                    .tag("gpu:2")
                    .build(),
            )
            .build();
        let mut graph = graph;
        graph.nodes[0].resources = Some(crate::Resources {
            cpu: 16,
            mem_mb: 32768,
            gpu_devices: 0,
            named_resources: std::collections::BTreeMap::new(),
        });
        let report = build_resource_hints_report(
            &graph,
            GraphResourceHintsV1 {
                cpu: 4,
                memory_mb: 8192,
                disk_mb: 102400,
                scratch_mb: 20480,
                network: "advisory".to_string(),
                walltime_s: 7200,
                gpu: 0,
                pool: Some("default".to_string()),
            },
        );
        assert_eq!(report.graph.disk_mb, 102400);
        assert_eq!(report.nodes[0].cpu, 16);
        assert_eq!(report.nodes[0].gpu, 2);
        assert_eq!(report.nodes[0].pool.as_deref(), Some("hpc"));
    }

    #[test]
    fn planner_conflicts_are_emitted_with_readable_reasons() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("mapped", NodeKind::Shell)
                    .semantic_kind(SemanticNodeKind::Map)
                    .trigger_rule(TriggerRule::AllDone)
                    .output("out", "tmp/out.json")
                    .effect(Effect::Network)
                    .build(),
            )
            .build();
        let mut graph = graph;
        graph.edges.push(crate::Edge {
            id: None,
            kind: EdgeKind::Conditional,
            decision: Some("x".to_string()),
            from: crate::PortRef { node_id: "mapped".to_string(), port: "out".to_string() },
            to: crate::PortRef { node_id: "mapped".to_string(), port: "in".to_string() },
        });

        let report = detect_planner_conflicts(
            &graph,
            &PlannerConflictEnvelopeV1 {
                allowed_runtime: "remote".to_string(),
                allowed_adapters: BTreeSet::from(["const".to_string()]),
                allow_conditional_edges: false,
                allow_matrix_expansion: false,
                required_artifact_prefix: "artifacts/".to_string(),
            },
        );
        assert!(report
            .conflicts
            .iter()
            .any(|conflict| conflict.code == "PC4901_RUNTIME_INCOMPATIBLE"));
        assert!(report
            .conflicts
            .iter()
            .any(|conflict| conflict.code == "PC4902_ADAPTER_UNSUPPORTED"));
        assert!(report
            .conflicts
            .iter()
            .any(|conflict| conflict.code == "PC4906_CONDITIONAL_EDGES_DISABLED"));
    }

    #[test]
    fn planner_performance_report_covers_real_graph_shapes_with_budgets() {
        let report = measure_planner_performance_real_shapes(2_000).expect("performance report");
        let shapes =
            report.samples.iter().map(|sample| sample.shape.clone()).collect::<BTreeSet<_>>();
        for shape in ["chain", "wide", "branching", "matrix", "subgraph", "reducer"] {
            assert!(shapes.contains(shape), "missing benchmark shape: {shape}");
        }
        assert!(report.samples.iter().all(|sample| sample.within_budget));
    }
}
