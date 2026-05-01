use serde::{Deserialize, Serialize};

/// Validation issue that must be actionable by operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateIssueV1 {
    pub code: String,
    pub path: String,
    pub message: String,
    pub remediation: String,
    pub severity: String,
}

/// Command contract for `dag validate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateCommandReportV1 {
    pub dag_path: String,
    pub ok: bool,
    pub strict_mode: bool,
    pub hard_failures: Vec<ValidateIssueV1>,
    pub lint_findings: Vec<ValidateIssueV1>,
}

/// Build a validate report with explicit fix guidance for every issue.
pub fn build_validate_command_report(
    dag_path: &str,
    strict_mode: bool,
    hard_failures: Vec<ValidateIssueV1>,
    lint_findings: Vec<ValidateIssueV1>,
) -> Result<ValidateCommandReportV1, String> {
    if dag_path.trim().is_empty() {
        return Err("dag_path must not be empty".to_string());
    }
    for issue in hard_failures.iter().chain(lint_findings.iter()) {
        for (field_name, field_value) in [
            ("code", issue.code.as_str()),
            ("path", issue.path.as_str()),
            ("message", issue.message.as_str()),
            ("remediation", issue.remediation.as_str()),
            ("severity", issue.severity.as_str()),
        ] {
            if field_value.trim().is_empty() {
                return Err(format!("validate issue {field_name} must not be empty"));
            }
        }
    }
    let ok = hard_failures.is_empty() && (!strict_mode || lint_findings.is_empty());
    Ok(ValidateCommandReportV1 {
        dag_path: dag_path.to_string(),
        ok,
        strict_mode,
        hard_failures,
        lint_findings,
    })
}

#[cfg(test)]
mod tests {
    use super::{build_validate_command_report, ValidateIssueV1};

    #[test]
    fn g071_validate_report_makes_invalid_graphs_fixable_from_output() {
        let report = build_validate_command_report(
            "workflows/pipeline.json",
            true,
            vec![ValidateIssueV1 {
                code: "VG1001".to_string(),
                path: "nodes[2].outputs[0].path".to_string(),
                message: "output path escapes run root".to_string(),
                remediation: "set output path under declared run root".to_string(),
                severity: "error".to_string(),
            }],
            vec![ValidateIssueV1 {
                code: "VG2002".to_string(),
                path: "nodes[1].params.timeout_ms".to_string(),
                message: "timeout is not explicitly set".to_string(),
                remediation: "set timeout_ms to a bounded value".to_string(),
                severity: "warn".to_string(),
            }],
        )
        .expect("validate report");

        assert!(!report.ok);
        assert_eq!(report.hard_failures.len(), 1);
        assert_eq!(
            report.hard_failures[0].remediation,
            "set output path under declared run root"
        );
        assert_eq!(report.lint_findings[0].severity, "warn");
    }
}
