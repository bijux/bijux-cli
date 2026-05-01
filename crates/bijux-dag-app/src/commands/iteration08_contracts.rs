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

/// One node row in a dry plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryPlanNodeRowV1 {
    pub node_id: String,
    pub expected_artifacts: Vec<String>,
    pub cache_eligible: bool,
}

/// Preflight check result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightCheckV1 {
    pub check: String,
    pub status: String,
    pub detail: String,
}

/// Command contract for `dag plan`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanCommandReportV1 {
    pub dag_path: String,
    pub plan_fingerprint: String,
    pub nodes: Vec<DryPlanNodeRowV1>,
    pub preflight_checks: Vec<PreflightCheckV1>,
}

/// Build a useful dry plan payload with cache and preflight visibility.
pub fn build_plan_command_report(
    dag_path: &str,
    plan_fingerprint: &str,
    nodes: Vec<DryPlanNodeRowV1>,
    preflight_checks: Vec<PreflightCheckV1>,
) -> Result<PlanCommandReportV1, String> {
    if dag_path.trim().is_empty() {
        return Err("dag_path must not be empty".to_string());
    }
    if plan_fingerprint.trim().is_empty() {
        return Err("plan_fingerprint must not be empty".to_string());
    }
    if nodes.is_empty() {
        return Err("plan must include at least one node".to_string());
    }
    if preflight_checks.is_empty() {
        return Err("preflight_checks must not be empty".to_string());
    }
    for node in &nodes {
        if node.node_id.trim().is_empty() {
            return Err("node_id must not be empty".to_string());
        }
        if node.expected_artifacts.is_empty() {
            return Err(format!("node {} must declare expected artifacts", node.node_id));
        }
    }
    for check in &preflight_checks {
        for (field_name, field_value) in [
            ("check", check.check.as_str()),
            ("status", check.status.as_str()),
            ("detail", check.detail.as_str()),
        ] {
            if field_value.trim().is_empty() {
                return Err(format!("preflight check {field_name} must not be empty"));
            }
        }
    }
    Ok(PlanCommandReportV1 {
        dag_path: dag_path.to_string(),
        plan_fingerprint: plan_fingerprint.to_string(),
        nodes,
        preflight_checks,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_plan_command_report, build_validate_command_report, DryPlanNodeRowV1, PreflightCheckV1,
        ValidateIssueV1,
    };

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

    #[test]
    fn g072_plan_report_exposes_fingerprint_artifacts_cache_and_preflight() {
        let report = build_plan_command_report(
            "workflows/pipeline.json",
            "plan-sha256-abc",
            vec![
                DryPlanNodeRowV1 {
                    node_id: "align-reads".to_string(),
                    expected_artifacts: vec!["outputs/aligned.bam".to_string()],
                    cache_eligible: true,
                },
                DryPlanNodeRowV1 {
                    node_id: "call-variants".to_string(),
                    expected_artifacts: vec!["outputs/variants.vcf".to_string()],
                    cache_eligible: false,
                },
            ],
            vec![
                PreflightCheckV1 {
                    check: "run_root_writable".to_string(),
                    status: "pass".to_string(),
                    detail: "run root exists and is writable".to_string(),
                },
                PreflightCheckV1 {
                    check: "adapter_shell_available".to_string(),
                    status: "pass".to_string(),
                    detail: "/bin/sh was found".to_string(),
                },
            ],
        )
        .expect("plan report");
        assert_eq!(report.plan_fingerprint, "plan-sha256-abc");
        assert_eq!(report.nodes.len(), 2);
        assert!(report.nodes.iter().any(|row| row.cache_eligible));
        assert_eq!(report.preflight_checks.len(), 2);
    }
}
