use serde::{Deserialize, Serialize};

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
    let required_kinds = [
        "const",
        "shell",
        "branch",
        "barrier",
        "reducer",
        "cacheable",
        "non-cacheable",
        "failure",
    ];
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

#[cfg(test)]
mod tests {
    use super::{
        build_graph_canonicalization_visibility_report, build_graph_examples_execution_report,
        build_surgical_validation_diagnostic, GraphExampleExecutionEntryV1,
    };

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
}
