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

#[cfg(test)]
mod tests {
    use super::{build_graph_examples_execution_report, GraphExampleExecutionEntryV1};

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
}
