use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    compile_graph, lint_graph, lower_graph_to_execution_plan, node_io_contract, parse_graph_strict,
    DagLintFinding, Graph, GraphError, Severity,
};

/// Per-example execution evidence used to prove examples are executable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphExampleExecutionEntryV1 {
    /// Example kind (`const`, `shell`, `branch`, `barrier`, `reducer`, `cacheable`, `non-cacheable`, `failure`).
    pub kind: String,
    /// Whether the example validated.
    pub validated: bool,
    /// Whether the example produced a plan.
    pub planned: bool,
    /// Whether the example executed locally.
    pub executed: bool,
    /// Whether the example is intentionally dry-plan only.
    pub dry_plan_only: bool,
}

/// Coverage report for executable graph examples.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphExamplesExecutionReportV1 {
    /// Example entries.
    pub entries: Vec<GraphExampleExecutionEntryV1>,
    /// Missing required example kinds.
    pub missing_kinds: Vec<String>,
    /// Whether all required example kinds are covered and executable/plannable.
    pub coverage_complete: bool,
}

/// Build graph example execution report with required-kind coverage checks.
pub fn build_graph_examples_execution_report(
    entries: Vec<GraphExampleExecutionEntryV1>,
) -> GraphExamplesExecutionReportV1 {
    let required_kinds =
        ["const", "shell", "branch", "barrier", "reducer", "cacheable", "non-cacheable", "failure"];
    let mut ordered = entries;
    ordered.sort_by(|left, right| left.kind.cmp(&right.kind));

    let missing_kinds: Vec<String> = required_kinds
        .iter()
        .filter(|kind| !ordered.iter().any(|entry| entry.kind == **kind))
        .map(|kind| (*kind).to_string())
        .collect();

    let entries_valid = ordered
        .iter()
        .all(|entry| entry.validated && entry.planned && (entry.executed || entry.dry_plan_only));

    GraphExamplesExecutionReportV1 {
        entries: ordered,
        missing_kinds: missing_kinds.clone(),
        coverage_complete: missing_kinds.is_empty() && entries_valid,
    }
}

/// Surgical validation diagnostic for graph authoring failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurgicalValidationDiagnosticV1 {
    /// Node identifier tied to the violation.
    pub node_id: String,
    /// Edge identifier when relevant.
    pub edge_id: Option<String>,
    /// Port name when relevant.
    pub port: Option<String>,
    /// Field path in graph source.
    pub field_path: String,
    /// Violated rule identifier.
    pub violated_rule: String,
    /// Severity (`error` or `warning`).
    pub severity: String,
    /// Human remediation instruction.
    pub remediation: String,
}

/// Build surgical validation diagnostic from explicit violation context.
pub fn build_surgical_validation_diagnostic(
    node_id: &str,
    edge_id: Option<&str>,
    port: Option<&str>,
    field_path: &str,
    violated_rule: &str,
    severity: &str,
    remediation: &str,
) -> Result<SurgicalValidationDiagnosticV1, String> {
    for (name, value) in [
        ("node_id", node_id),
        ("field_path", field_path),
        ("violated_rule", violated_rule),
        ("severity", severity),
        ("remediation", remediation),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{name} cannot be empty"));
        }
    }
    if !matches!(severity, "error" | "warning") {
        return Err("severity must be `error` or `warning`".to_string());
    }
    Ok(SurgicalValidationDiagnosticV1 {
        node_id: node_id.to_string(),
        edge_id: edge_id.map(ToString::to_string),
        port: port.map(ToString::to_string),
        field_path: field_path.to_string(),
        violated_rule: violated_rule.to_string(),
        severity: severity.to_string(),
        remediation: remediation.to_string(),
    })
}

/// Canonicalization visibility report for authored graph surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphCanonicalizationVisibilityReportV1 {
    /// Canonical JSON representation.
    pub canonical_json: String,
    /// Deterministic graph fingerprint.
    pub graph_fingerprint: String,
    /// Per-node fingerprints for explainability.
    pub node_fingerprints: Vec<(String, String)>,
}

/// Build canonicalization visibility report for user-facing diagnostics.
pub fn build_graph_canonicalization_visibility_report(
    canonical_json: &str,
    graph_fingerprint: &str,
    node_fingerprints: Vec<(String, String)>,
) -> Result<GraphCanonicalizationVisibilityReportV1, String> {
    if canonical_json.trim().is_empty() {
        return Err("canonical_json cannot be empty".to_string());
    }
    if graph_fingerprint.trim().is_empty() {
        return Err("graph_fingerprint cannot be empty".to_string());
    }
    if node_fingerprints.is_empty() {
        return Err("node_fingerprints cannot be empty".to_string());
    }
    Ok(GraphCanonicalizationVisibilityReportV1 {
        canonical_json: canonical_json.to_string(),
        graph_fingerprint: graph_fingerprint.to_string(),
        node_fingerprints,
    })
}

/// Strict parity report between programmatic builder and file-authored graphs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphBuilderParityReportV1 {
    /// Whether canonical JSON bytes are identical.
    pub canonical_json_equal: bool,
    /// Whether graph fingerprints are identical.
    pub graph_fingerprint_equal: bool,
    /// Whether planner fingerprints are identical.
    pub planner_fingerprint_equal: bool,
    /// Validation error codes emitted by builder-authored graph.
    pub builder_validation_error_codes: Vec<String>,
    /// Validation error codes emitted by file-authored graph.
    pub file_validation_error_codes: Vec<String>,
    /// Human-readable mismatch descriptions.
    pub mismatches: Vec<String>,
}

/// Ensure builder and file-authored graphs share strict canonical and planning parity.
pub fn build_graph_builder_parity_report(
    builder_graph: &Graph,
    file_graph_json: &str,
) -> Result<GraphBuilderParityReportV1, GraphError> {
    let file_graph = parse_graph_strict(file_graph_json)?;
    let builder_compiled = compile_graph(builder_graph)?;
    let file_compiled = compile_graph(&file_graph)?;

    let builder_validation_error_codes = builder_compiled
        .diagnostics
        .iter()
        .filter(|diag| diag.severity == Severity::Error)
        .map(|diag| diag.code.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let file_validation_error_codes = file_compiled
        .diagnostics
        .iter()
        .filter(|diag| diag.severity == Severity::Error)
        .map(|diag| diag.code.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let builder_canonical_json = builder_compiled.normalized_graph.to_canonical_json()?;
    let file_canonical_json = file_compiled.normalized_graph.to_canonical_json()?;
    let canonical_json_equal = builder_canonical_json == file_canonical_json;
    let graph_fingerprint_equal =
        builder_compiled.graph_fingerprint == file_compiled.graph_fingerprint;

    let builder_plan =
        lower_graph_to_execution_plan(&builder_compiled.normalized_graph, Default::default())
            .map_err(|_| GraphError::ValidationFailed)?;
    let file_plan =
        lower_graph_to_execution_plan(&file_compiled.normalized_graph, Default::default())
            .map_err(|_| GraphError::ValidationFailed)?;
    let planner_fingerprint_equal =
        builder_plan.planner_fingerprint == file_plan.planner_fingerprint;

    let mut mismatches = Vec::new();
    if !builder_validation_error_codes.is_empty() || !file_validation_error_codes.is_empty() {
        mismatches.push("builder and file graphs must both pass semantic validation".to_string());
    }
    if !canonical_json_equal {
        mismatches.push("canonical JSON bytes diverged between authoring surfaces".to_string());
    }
    if !graph_fingerprint_equal {
        mismatches.push("graph fingerprints diverged between authoring surfaces".to_string());
    }
    if !planner_fingerprint_equal {
        mismatches.push("planner fingerprints diverged between authoring surfaces".to_string());
    }

    Ok(GraphBuilderParityReportV1 {
        canonical_json_equal,
        graph_fingerprint_equal,
        planner_fingerprint_equal,
        builder_validation_error_codes,
        file_validation_error_codes,
        mismatches,
    })
}

/// Refusal item for cycle/reachability safety checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphRefusalV1 {
    /// Stable refusal code.
    pub code: String,
    /// Violating subject (`node:<id>`, `edge:<from>:<to>`, `artifact:<name>`).
    pub subject: String,
    /// Human-readable remediation.
    pub remediation: String,
}

/// Complete cycle and reachability refusal report for required graph surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleReachabilityRejectionReportV1 {
    /// Ordered refusal list.
    pub refusals: Vec<GraphRefusalV1>,
}

/// Build cycle/reachability refusal report with explicit codes and remediations.
pub fn build_cycle_reachability_rejection_report(
    graph: &Graph,
    required_nodes: &BTreeSet<String>,
    required_artifacts: &BTreeSet<String>,
) -> CycleReachabilityRejectionReportV1 {
    let mut refusals = Vec::new();

    for edge in &graph.edges {
        if edge.from.node_id == edge.to.node_id {
            refusals.push(GraphRefusalV1 {
                code: "R2101_SELF_CYCLE".to_string(),
                subject: format!(
                    "edge:{}:{}->{}:{}",
                    edge.from.node_id, edge.from.port, edge.to.node_id, edge.to.port
                ),
                remediation: "remove self-loop or insert an intermediate synchronization node"
                    .to_string(),
            });
        }
    }

    if graph.has_cycle() && !refusals.iter().any(|entry| entry.code == "R2101_SELF_CYCLE") {
        refusals.push(GraphRefusalV1 {
            code: "R2102_INDIRECT_CYCLE".to_string(),
            subject: "graph".to_string(),
            remediation: "break indirect dependency loops so a full topological order exists"
                .to_string(),
        });
    }

    let mut outgoing = BTreeMap::<String, Vec<String>>::new();
    for edge in &graph.edges {
        outgoing.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    }
    let mut visited = BTreeSet::new();
    let mut stack = graph
        .nodes
        .iter()
        .filter(|node| node.inputs.is_empty())
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    while let Some(node_id) = stack.pop() {
        if !visited.insert(node_id.clone()) {
            continue;
        }
        if let Some(next_nodes) = outgoing.get(&node_id) {
            stack.extend(next_nodes.iter().cloned());
        }
    }

    for node_id in required_nodes {
        if !visited.contains(node_id) {
            refusals.push(GraphRefusalV1 {
                code: "R2103_UNREACHABLE_REQUIRED_NODE".to_string(),
                subject: format!("node:{node_id}"),
                remediation:
                    "add an upstream path from graph inputs or remove required designation"
                        .to_string(),
            });
        }
    }

    let mut produced_artifacts = BTreeSet::new();
    let mut consumed_artifacts = BTreeSet::new();
    for node in &graph.nodes {
        for output in &node.outputs {
            produced_artifacts.insert(format!("{}.{}", node.id, output.name));
        }
    }
    for edge in &graph.edges {
        consumed_artifacts.insert(format!("{}.{}", edge.from.node_id, edge.from.port));
    }

    for artifact in produced_artifacts.difference(&consumed_artifacts) {
        if !required_artifacts.contains(artifact) {
            refusals.push(GraphRefusalV1 {
                code: "R2104_ORPHAN_OUTPUT".to_string(),
                subject: format!("artifact:{artifact}"),
                remediation: "connect the output to downstream inputs or declare it required"
                    .to_string(),
            });
        }
    }

    for artifact in required_artifacts {
        if !produced_artifacts.contains(artifact) {
            refusals.push(GraphRefusalV1 {
                code: "R2105_DEAD_END_REQUIRED_ARTIFACT".to_string(),
                subject: format!("artifact:{artifact}"),
                remediation:
                    "add a producing node output for the required artifact or remove requirement"
                        .to_string(),
            });
        }
    }

    refusals.sort_by(|left, right| {
        left.code.cmp(&right.code).then_with(|| left.subject.cmp(&right.subject))
    });
    refusals.dedup();

    CycleReachabilityRejectionReportV1 { refusals }
}

/// Node output contract hint for user-facing port contract reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePortOutputHintV1 {
    /// Output name.
    pub name: String,
    /// Output path.
    pub path: String,
    /// Whether output is required.
    pub required: bool,
    /// MIME-like media hint.
    pub media_type: String,
}

/// Node-level port contract snapshot with required inputs and outputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePortContractSnapshotV1 {
    /// Node identifier.
    pub node_id: String,
    /// Declared required input ports.
    pub required_inputs: Vec<String>,
    /// Declared outputs with requirement/type hints.
    pub outputs: Vec<NodePortOutputHintV1>,
}

/// Port contract violation with deterministic code and subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortContractViolationV1 {
    /// Stable violation code.
    pub code: String,
    /// Subject key.
    pub subject: String,
}

/// Deterministic port contract enforcement report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortContractEnforcementReportV1 {
    /// Node port contract snapshots.
    pub node_contracts: Vec<NodePortContractSnapshotV1>,
    /// Violations ordered by code/subject.
    pub violations: Vec<PortContractViolationV1>,
}

/// Build deterministic port contract report for unknown, duplicate, missing, and unused ports.
pub fn build_port_contract_enforcement_report(graph: &Graph) -> PortContractEnforcementReportV1 {
    let mut node_contracts = graph
        .nodes
        .iter()
        .map(|node| {
            let outputs = node_io_contract(graph, &node.id)
                .map(|io| {
                    io.outputs
                        .into_iter()
                        .map(|output| NodePortOutputHintV1 {
                            name: output.name,
                            path: output.path,
                            required: output.required,
                            media_type: output.media_type,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            NodePortContractSnapshotV1 {
                node_id: node.id.clone(),
                required_inputs: node.inputs.clone(),
                outputs,
            }
        })
        .collect::<Vec<_>>();
    node_contracts.sort_by(|left, right| left.node_id.cmp(&right.node_id));

    let mut violations = Vec::new();
    let mut duplicate_bindings = BTreeSet::new();

    for edge in &graph.edges {
        let from_node = graph.nodes.iter().find(|node| node.id == edge.from.node_id);
        let to_node = graph.nodes.iter().find(|node| node.id == edge.to.node_id);

        let from_port_known = from_node
            .map(|node| node.outputs.iter().any(|output| output.name == edge.from.port))
            .unwrap_or(false);
        if !from_port_known {
            violations.push(PortContractViolationV1 {
                code: "P3001_UNKNOWN_OUTPUT_PORT".to_string(),
                subject: format!("{}.{}", edge.from.node_id, edge.from.port),
            });
        }

        let to_port_known = to_node
            .map(|node| node.inputs.iter().any(|input| input == &edge.to.port))
            .unwrap_or(false);
        if !to_port_known {
            violations.push(PortContractViolationV1 {
                code: "P3002_UNKNOWN_INPUT_PORT".to_string(),
                subject: format!("{}.{}", edge.to.node_id, edge.to.port),
            });
        }

        let binding = format!("{}.{}", edge.to.node_id, edge.to.port);
        if !duplicate_bindings.insert(binding.clone()) {
            violations.push(PortContractViolationV1 {
                code: "P3003_DUPLICATE_INPUT_BINDING".to_string(),
                subject: binding,
            });
        }
    }

    for node in &graph.nodes {
        for input in &node.inputs {
            let has_binding =
                graph.edges.iter().any(|edge| edge.to.node_id == node.id && edge.to.port == *input);
            if !has_binding {
                violations.push(PortContractViolationV1 {
                    code: "P3004_MISSING_REQUIRED_INPUT".to_string(),
                    subject: format!("{}.{}", node.id, input),
                });
            }
        }

        for output in &node.outputs {
            let used = graph
                .edges
                .iter()
                .any(|edge| edge.from.node_id == node.id && edge.from.port == output.name);
            if !used {
                violations.push(PortContractViolationV1 {
                    code: "P3005_UNUSED_DECLARED_OUTPUT".to_string(),
                    subject: format!("{}.{}", node.id, output.name),
                });
            }
        }
    }

    violations.sort_by(|left, right| {
        left.code.cmp(&right.code).then_with(|| left.subject.cmp(&right.subject))
    });
    violations.dedup();

    PortContractEnforcementReportV1 { node_contracts, violations }
}

/// Graph package file entry used in import validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPackageFileEntryV1 {
    /// Normalized relative package path.
    pub path: String,
    /// SHA-256 digest in lowercase hex.
    pub sha256: String,
}

/// Graph package bundle import payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPackageBundleV1 {
    /// Bundle format version.
    pub format_version: String,
    /// Graph payload.
    pub graph: Graph,
    /// Config/schema descriptors.
    pub schema_versions: Vec<String>,
    /// Required produced artifacts as `node.output`.
    pub expected_artifacts: Vec<String>,
    /// Example identifiers included with the package.
    pub examples: Vec<String>,
    /// File integrity manifest.
    pub files: Vec<GraphPackageFileEntryV1>,
}

/// Import refusal for graph package bundles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPackageImportRefusalV1 {
    /// Stable refusal code.
    pub code: String,
    /// Refusal subject.
    pub subject: String,
    /// Remediation guidance.
    pub remediation: String,
}

/// Graph package import validation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPackageImportReportV1 {
    /// Whether bundle is safe and current for import.
    pub accepted: bool,
    /// Refusal list.
    pub refusals: Vec<GraphPackageImportRefusalV1>,
}

/// Validate graph package import constraints and return deterministic refusal detail.
pub fn validate_graph_package_import(
    bundle: &GraphPackageBundleV1,
    expected_graph_spec: &str,
) -> GraphPackageImportReportV1 {
    let mut refusals = Vec::new();
    if bundle.format_version != "bijux-graph-package/v1" {
        refusals.push(GraphPackageImportRefusalV1 {
            code: "B4001_CORRUPT_BUNDLE".to_string(),
            subject: "format_version".to_string(),
            remediation: "export package again using bijux-graph-package/v1".to_string(),
        });
    }
    if bundle.graph.spec != expected_graph_spec {
        refusals.push(GraphPackageImportRefusalV1 {
            code: "B4002_STALE_SPEC".to_string(),
            subject: format!("graph.spec={}", bundle.graph.spec),
            remediation: format!("migrate graph spec to {expected_graph_spec} before import"),
        });
    }
    if compile_graph(&bundle.graph).is_err() {
        refusals.push(GraphPackageImportRefusalV1 {
            code: "B4001_CORRUPT_BUNDLE".to_string(),
            subject: "graph".to_string(),
            remediation: "fix graph validation errors before packaging".to_string(),
        });
    }

    let mut seen_paths = BTreeSet::new();
    for entry in &bundle.files {
        let normalized = entry.path.replace('\\', "/");
        let unsafe_path = normalized.starts_with('/')
            || normalized.contains("..")
            || normalized.is_empty()
            || normalized.contains("//");
        if unsafe_path || !seen_paths.insert(normalized.clone()) {
            refusals.push(GraphPackageImportRefusalV1 {
                code: "B4003_UNSAFE_PATH".to_string(),
                subject: entry.path.clone(),
                remediation: "use unique normalized relative paths inside package files"
                    .to_string(),
            });
        }
        let digest_valid =
            entry.sha256.len() == 64 && entry.sha256.chars().all(|char| char.is_ascii_hexdigit());
        if !digest_valid {
            refusals.push(GraphPackageImportRefusalV1 {
                code: "B4004_INVALID_FILE_DIGEST".to_string(),
                subject: entry.path.clone(),
                remediation: "recompute file sha256 digests and repack bundle".to_string(),
            });
        }
    }

    let produced = bundle
        .graph
        .nodes
        .iter()
        .flat_map(|node| {
            node.outputs
                .iter()
                .map(move |output| format!("{}.{}", node.id, output.name))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();
    for artifact in &bundle.expected_artifacts {
        if !produced.contains(artifact) {
            refusals.push(GraphPackageImportRefusalV1 {
                code: "B4005_MISSING_EXPECTED_ARTIFACT".to_string(),
                subject: artifact.clone(),
                remediation: "align expected_artifacts with declared node outputs".to_string(),
            });
        }
    }

    refusals.sort_by(|left, right| {
        left.code.cmp(&right.code).then_with(|| left.subject.cmp(&right.subject))
    });
    refusals.dedup();
    GraphPackageImportReportV1 { accepted: refusals.is_empty(), refusals }
}

/// Refusal details for graph migration attempts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphMigrationRefusalV1 {
    /// Stable refusal code.
    pub code: String,
    /// Reason text.
    pub reason: String,
    /// Remediation guidance.
    pub remediation: String,
}

/// User-facing migration preview surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMigrationPreviewV1 {
    /// Source spec version.
    pub from_spec: String,
    /// Target spec version.
    pub to_spec: String,
    /// Deterministic list of previewed changes.
    pub changes: Vec<String>,
    /// Whether migration is safe to apply.
    pub safe_to_apply: bool,
    /// Refusal details for unsafe migrations.
    pub refusal: Option<GraphMigrationRefusalV1>,
    /// Migrated graph when safe.
    pub migrated_graph: Option<Graph>,
}

/// Preview graph migration changes or produce an explicit unsafe refusal.
pub fn preview_graph_migration(graph: &Graph, target_spec: &str) -> GraphMigrationPreviewV1 {
    let mut changes = Vec::new();
    if target_spec.trim().is_empty() {
        return GraphMigrationPreviewV1 {
            from_spec: graph.spec.clone(),
            to_spec: target_spec.to_string(),
            changes,
            safe_to_apply: false,
            refusal: Some(GraphMigrationRefusalV1 {
                code: "M5001_INVALID_TARGET_SPEC".to_string(),
                reason: "target spec is empty".to_string(),
                remediation: "provide a non-empty target spec".to_string(),
            }),
            migrated_graph: None,
        };
    }

    let from = graph.spec.as_str();
    let migratable_alias = matches!(from, "0.1" | "v0.1");
    let already_target = from == target_spec;
    let target_supported = target_spec == crate::SPEC_VERSION;
    if !already_target && !migratable_alias {
        return GraphMigrationPreviewV1 {
            from_spec: graph.spec.clone(),
            to_spec: target_spec.to_string(),
            changes,
            safe_to_apply: false,
            refusal: Some(GraphMigrationRefusalV1 {
                code: "M5002_UNSUPPORTED_SOURCE_SPEC".to_string(),
                reason: format!("source spec {} is not recognized for safe migration", graph.spec),
                remediation:
                    "migrate through supported bridge versions or regenerate graph from current schema"
                        .to_string(),
            }),
            migrated_graph: None,
        };
    }
    if !target_supported {
        return GraphMigrationPreviewV1 {
            from_spec: graph.spec.clone(),
            to_spec: target_spec.to_string(),
            changes,
            safe_to_apply: false,
            refusal: Some(GraphMigrationRefusalV1 {
                code: "M5003_UNSUPPORTED_TARGET_SPEC".to_string(),
                reason: format!("target spec {target_spec} is not supported"),
                remediation: format!("target spec must be {}", crate::SPEC_VERSION),
            }),
            migrated_graph: None,
        };
    }

    let mut migrated = graph.clone();
    if migratable_alias {
        migrated.spec = target_spec.to_string();
        changes.push(format!("normalize graph spec {} -> {}", graph.spec, target_spec));
    }
    if migrated.meta.is_none() {
        changes.push("meta remains absent; no default owners introduced".to_string());
    }
    if changes.is_empty() {
        changes.push("graph already matches current schema contract".to_string());
    }

    GraphMigrationPreviewV1 {
        from_spec: graph.spec.clone(),
        to_spec: target_spec.to_string(),
        changes,
        safe_to_apply: true,
        refusal: None,
        migrated_graph: Some(migrated),
    }
}

/// Apply graph migration when preview indicates a safe transition.
pub fn apply_graph_migration(
    graph: &Graph,
    target_spec: &str,
) -> Result<Graph, GraphMigrationRefusalV1> {
    let preview = preview_graph_migration(graph, target_spec);
    if preview.safe_to_apply {
        return Ok(preview.migrated_graph.expect("safe migration should include graph"));
    }
    Err(preview.refusal.expect("unsafe migration should include refusal"))
}

/// Policy that promotes selected lint codes to blocking errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintPromotionPolicyV1 {
    /// Lint codes to promote to blocking status.
    pub promoted_codes: BTreeSet<String>,
}

/// Semantic lint evaluation report that separates advisory and blocking surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticLintEvaluationReportV1 {
    /// Hard validation error codes.
    pub hard_validation_errors: Vec<String>,
    /// Advisory lint findings that remain non-blocking.
    pub advisory_lints: Vec<DagLintFinding>,
    /// Lints promoted to blocking by policy.
    pub blocking_lints: Vec<DagLintFinding>,
    /// Whether the graph is eligible for execution.
    pub can_execute: bool,
}

/// Evaluate semantic lint in non-blocking mode unless policy promotion is requested.
pub fn evaluate_semantic_lint(
    graph: &Graph,
    policy: Option<&LintPromotionPolicyV1>,
) -> SemanticLintEvaluationReportV1 {
    let hard_validation_errors = graph
        .validate_with_warnings()
        .into_iter()
        .filter(|diag| diag.severity == Severity::Error)
        .map(|diag| diag.code)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let promotion_codes = policy.map(|value| value.promoted_codes.clone()).unwrap_or_default();

    let mut advisory_lints = Vec::new();
    let mut blocking_lints = Vec::new();
    for lint in lint_graph(graph) {
        if promotion_codes.contains(&lint.code) {
            blocking_lints.push(lint);
        } else {
            advisory_lints.push(lint);
        }
    }

    SemanticLintEvaluationReportV1 {
        can_execute: hard_validation_errors.is_empty() && blocking_lints.is_empty(),
        hard_validation_errors,
        advisory_lints,
        blocking_lints,
    }
}

/// Authoring convergence report across JSON/file and builder styles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphAuthoringConvergenceReportV1 {
    /// Validation error codes from JSON/file authoring.
    pub json_validation_error_codes: Vec<String>,
    /// Validation error codes from builder authoring.
    pub builder_validation_error_codes: Vec<String>,
    /// JSON/file graph fingerprint.
    pub json_graph_fingerprint: String,
    /// Builder graph fingerprint.
    pub builder_graph_fingerprint: String,
    /// JSON/file planner fingerprint.
    pub json_planner_fingerprint: String,
    /// Builder planner fingerprint.
    pub builder_planner_fingerprint: String,
    /// Whether all surfaces converge.
    pub converged: bool,
    /// Divergence list when not converged.
    pub divergences: Vec<String>,
}

/// Validate that JSON/file and builder authoring converge to one semantic model.
pub fn build_graph_authoring_convergence_report(
    json_graph: &str,
    builder_graph: &Graph,
) -> Result<GraphAuthoringConvergenceReportV1, GraphError> {
    let parsed = parse_graph_strict(json_graph)?;
    let json_compiled = compile_graph(&parsed)?;
    let builder_compiled = compile_graph(builder_graph)?;

    let json_validation_error_codes = json_compiled
        .diagnostics
        .iter()
        .filter(|diag| diag.severity == Severity::Error)
        .map(|diag| diag.code.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let builder_validation_error_codes = builder_compiled
        .diagnostics
        .iter()
        .filter(|diag| diag.severity == Severity::Error)
        .map(|diag| diag.code.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let json_plan =
        lower_graph_to_execution_plan(&json_compiled.normalized_graph, Default::default())
            .map_err(|_| GraphError::ValidationFailed)?;
    let builder_plan =
        lower_graph_to_execution_plan(&builder_compiled.normalized_graph, Default::default())
            .map_err(|_| GraphError::ValidationFailed)?;

    let mut divergences = Vec::new();
    if json_validation_error_codes != builder_validation_error_codes {
        divergences.push("validation error code sets differ across authoring surfaces".to_string());
    }
    if json_compiled.graph_fingerprint != builder_compiled.graph_fingerprint {
        divergences.push("graph fingerprints differ across authoring surfaces".to_string());
    }
    if json_plan.planner_fingerprint != builder_plan.planner_fingerprint {
        divergences.push("planner fingerprints differ across authoring surfaces".to_string());
    }

    Ok(GraphAuthoringConvergenceReportV1 {
        json_validation_error_codes,
        builder_validation_error_codes,
        json_graph_fingerprint: json_compiled.graph_fingerprint,
        builder_graph_fingerprint: builder_compiled.graph_fingerprint,
        json_planner_fingerprint: json_plan.planner_fingerprint,
        builder_planner_fingerprint: builder_plan.planner_fingerprint,
        converged: divergences.is_empty(),
        divergences,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        apply_graph_migration, build_cycle_reachability_rejection_report,
        build_graph_authoring_convergence_report, build_graph_builder_parity_report,
        build_graph_canonicalization_visibility_report, build_graph_examples_execution_report,
        build_port_contract_enforcement_report, build_surgical_validation_diagnostic,
        evaluate_semantic_lint, preview_graph_migration, validate_graph_package_import,
        GraphExampleExecutionEntryV1, GraphPackageBundleV1, GraphPackageFileEntryV1,
        LintPromotionPolicyV1,
    };
    use crate::{DagBuilder, Edge, EdgeKind, Effect, NodeBuilder, NodeKind, PortRef};
    use std::collections::BTreeSet;

    #[test]
    fn g021_graph_examples_execution_report_requires_all_example_kinds() {
        let report = build_graph_examples_execution_report(vec![
            GraphExampleExecutionEntryV1 {
                kind: "const".to_string(),
                validated: true,
                planned: true,
                executed: true,
                dry_plan_only: false,
            },
            GraphExampleExecutionEntryV1 {
                kind: "shell".to_string(),
                validated: true,
                planned: true,
                executed: true,
                dry_plan_only: false,
            },
        ]);
        assert!(!report.coverage_complete);
        assert!(report.missing_kinds.contains(&"branch".to_string()));
    }

    #[test]
    fn g022_surgical_validation_diagnostic_contains_violation_context() {
        let diagnostic = build_surgical_validation_diagnostic(
            "node.validate",
            Some("edge.12"),
            Some("output"),
            "nodes[3].outputs[0]",
            "unknown_port_reference",
            "error",
            "align edge source port to declared node outputs",
        )
        .expect("diagnostic should build");
        assert_eq!(diagnostic.node_id, "node.validate");
        assert_eq!(diagnostic.edge_id.as_deref(), Some("edge.12"));
    }

    #[test]
    fn g023_canonicalization_visibility_report_exposes_graph_and_node_fingerprints() {
        let report = build_graph_canonicalization_visibility_report(
            "{\"graph_id\":\"demo\"}",
            "sha256:graph-demo",
            vec![("node.a".to_string(), "sha256:node-a".to_string())],
        )
        .expect("canonicalization visibility report should build");
        assert_eq!(report.graph_fingerprint, "sha256:graph-demo");
        assert_eq!(report.node_fingerprints.len(), 1);
    }

    #[test]
    fn g024_builder_and_file_authoring_surfaces_match_canonical_and_plan_identity() {
        let builder_graph = DagBuilder::new()
            .node(
                NodeBuilder::new("seed", NodeKind::Const)
                    .output("out", "artifacts/seed.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("produce", NodeKind::Shell)
                    .input("seed")
                    .output("out", "artifacts/produce.json")
                    .effect(Effect::Filesystem)
                    .build(),
            )
            .node(
                NodeBuilder::new("consume", NodeKind::Shell)
                    .input("in")
                    .output("done", "artifacts/consume.json")
                    .effect(Effect::Filesystem)
                    .build(),
            )
            .edge("seed", "out", "produce", "seed")
            .edge("produce", "out", "consume", "in")
            .build();

        let file_graph_json = serde_json::to_string_pretty(&builder_graph)
            .expect("graph should serialize to file-authored JSON");
        let report =
            build_graph_builder_parity_report(&builder_graph, &file_graph_json).expect("parity");

        assert!(report.builder_validation_error_codes.is_empty());
        assert!(report.file_validation_error_codes.is_empty());
        assert!(report.canonical_json_equal);
        assert!(report.graph_fingerprint_equal);
        assert!(report.planner_fingerprint_equal);
        assert!(report.mismatches.is_empty());
    }

    #[test]
    fn g025_cycle_and_reachability_rejections_have_exact_codes_and_remediation() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("start", NodeKind::Const)
                    .output("out", "artifacts/start.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("self_loop", NodeKind::Const)
                    .input("in")
                    .output("out", "artifacts/self_loop.json")
                    .build(),
            )
            .edge("start", "out", "self_loop", "in")
            .build();

        let mut graph_with_cycle = graph;
        graph_with_cycle.edges.push(Edge {
            id: Some("self".to_string()),
            kind: EdgeKind::Data,
            decision: None,
            from: PortRef { node_id: "self_loop".to_string(), port: "out".to_string() },
            to: PortRef { node_id: "self_loop".to_string(), port: "in".to_string() },
        });

        let required_nodes = BTreeSet::from(["must_run".to_string()]);
        let required_artifacts = BTreeSet::from(["must_run.out".to_string()]);
        let report = build_cycle_reachability_rejection_report(
            &graph_with_cycle,
            &required_nodes,
            &required_artifacts,
        );
        let refusal_codes =
            report.refusals.iter().map(|entry| entry.code.clone()).collect::<BTreeSet<_>>();
        assert!(refusal_codes.contains("R2101_SELF_CYCLE"));
        assert!(refusal_codes.contains("R2103_UNREACHABLE_REQUIRED_NODE"));
        assert!(refusal_codes.contains("R2105_DEAD_END_REQUIRED_ARTIFACT"));
        assert!(report.refusals.iter().all(|entry| !entry.remediation.trim().is_empty()));
    }

    #[test]
    fn g026_port_contract_enforcement_is_deterministic_for_unknown_duplicate_and_missing_ports() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("source", NodeKind::Const)
                    .output("out", "artifacts/source.json")
                    .output("ghost", "artifacts/ghost.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("sink", NodeKind::Const)
                    .input("input_a")
                    .input("input_b")
                    .output("done", "artifacts/sink.json")
                    .build(),
            )
            .edge("source", "out", "sink", "input_a")
            .edge("source", "out", "sink", "input_a")
            .edge("source", "missing_output", "sink", "input_c")
            .build();

        let report = build_port_contract_enforcement_report(&graph);
        let codes = report
            .violations
            .iter()
            .map(|violation| violation.code.clone())
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("P3001_UNKNOWN_OUTPUT_PORT"));
        assert!(codes.contains("P3002_UNKNOWN_INPUT_PORT"));
        assert!(codes.contains("P3003_DUPLICATE_INPUT_BINDING"));
        assert!(codes.contains("P3004_MISSING_REQUIRED_INPUT"));
        assert!(codes.contains("P3005_UNUSED_DECLARED_OUTPUT"));
    }

    #[test]
    fn g027_graph_package_import_refuses_corrupt_stale_and_unsafe_bundles() {
        let graph = DagBuilder::new()
            .node(
                NodeBuilder::new("producer", NodeKind::Const)
                    .output("out", "artifacts/out.json")
                    .build(),
            )
            .build();
        let bundle = GraphPackageBundleV1 {
            format_version: "bijux-graph-package/v0".to_string(),
            graph: graph.clone(),
            schema_versions: vec!["schema/v1".to_string()],
            expected_artifacts: vec!["producer.missing".to_string()],
            examples: vec!["minimal".to_string()],
            files: vec![GraphPackageFileEntryV1 {
                path: "../escape.json".to_string(),
                sha256: "abc".to_string(),
            }],
        };

        let report = validate_graph_package_import(&bundle, crate::SPEC_VERSION);
        let codes = report.refusals.iter().map(|entry| entry.code.clone()).collect::<BTreeSet<_>>();
        assert!(!report.accepted);
        assert!(codes.contains("B4001_CORRUPT_BUNDLE"));
        assert!(codes.contains("B4003_UNSAFE_PATH"));
        assert!(codes.contains("B4004_INVALID_FILE_DIGEST"));
        assert!(codes.contains("B4005_MISSING_EXPECTED_ARTIFACT"));
    }

    #[test]
    fn g028_graph_migration_preview_and_apply_are_user_visible() {
        let legacy_graph_json = r#"
        {
          "spec": "v0.1",
          "nodes": [
            {
              "id": "step",
              "kind": "const",
              "inputs": [],
              "outputs": [{"name":"out","path":"artifacts/out.json"}]
            }
          ],
          "edges": []
        }
        "#;
        let legacy_graph =
            crate::parse_graph_strict(legacy_graph_json).expect("legacy graph parse");
        let preview = preview_graph_migration(&legacy_graph, crate::SPEC_VERSION);
        assert!(preview.safe_to_apply);
        assert!(!preview.changes.is_empty());

        let migrated = apply_graph_migration(&legacy_graph, crate::SPEC_VERSION).expect("migrate");
        assert_eq!(migrated.spec, crate::SPEC_VERSION);
    }

    #[test]
    fn g029_semantic_lint_is_advisory_unless_promoted_by_policy() {
        let lint_only_graph =
            DagBuilder::new().node(NodeBuilder::new("n1", NodeKind::Const).build()).build();

        let advisory = evaluate_semantic_lint(&lint_only_graph, None);
        assert!(advisory.hard_validation_errors.is_empty());
        assert!(advisory.can_execute);
        assert!(advisory
            .advisory_lints
            .iter()
            .any(|finding| finding.code == "LINT_OUTPUT_MISSING"));

        let policy = LintPromotionPolicyV1 {
            promoted_codes: BTreeSet::from(["LINT_OUTPUT_MISSING".to_string()]),
        };
        let promoted = evaluate_semantic_lint(&lint_only_graph, Some(&policy));
        assert!(!promoted.can_execute);
        assert!(promoted
            .blocking_lints
            .iter()
            .any(|finding| finding.code == "LINT_OUTPUT_MISSING"));
    }

    #[test]
    fn g030_json_and_builder_authoring_converge_to_single_semantic_model() {
        let builder_graph = DagBuilder::new()
            .node(
                NodeBuilder::new("extract", NodeKind::Const)
                    .output("out", "artifacts/extract.json")
                    .build(),
            )
            .node(
                NodeBuilder::new("load", NodeKind::Shell)
                    .input("in")
                    .output("done", "artifacts/load.json")
                    .effect(Effect::Filesystem)
                    .build(),
            )
            .edge("extract", "out", "load", "in")
            .build();
        let json_graph = serde_json::to_string_pretty(&builder_graph).expect("serialize");

        let report =
            build_graph_authoring_convergence_report(&json_graph, &builder_graph).expect("report");
        assert!(report.json_validation_error_codes.is_empty());
        assert!(report.builder_validation_error_codes.is_empty());
        assert!(report.converged);
        assert!(report.divergences.is_empty());
    }
}
