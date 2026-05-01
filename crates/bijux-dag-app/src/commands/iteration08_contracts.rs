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

/// Minimal run summary for operator-visible `dag run` results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCommandReportV1 {
    pub dag_path: String,
    pub run_id: String,
    pub run_state: String,
    pub logs_path: String,
    pub artifacts_root: String,
    pub verification_path: String,
}

/// Build `dag run` report proving logs, artifacts, state, and verification path are visible.
pub fn build_run_command_report(
    dag_path: &str,
    run_id: &str,
    run_state: &str,
    logs_path: &str,
    artifacts_root: &str,
    verification_path: &str,
) -> Result<RunCommandReportV1, String> {
    for (field_name, field_value) in [
        ("dag_path", dag_path),
        ("run_id", run_id),
        ("run_state", run_state),
        ("logs_path", logs_path),
        ("artifacts_root", artifacts_root),
        ("verification_path", verification_path),
    ] {
        if field_value.trim().is_empty() {
            return Err(format!("{field_name} must not be empty"));
        }
    }
    if !matches!(run_state, "completed" | "failed" | "cancelled") {
        return Err("run_state must be completed, failed, or cancelled".to_string());
    }
    Ok(RunCommandReportV1 {
        dag_path: dag_path.to_string(),
        run_id: run_id.to_string(),
        run_state: run_state.to_string(),
        logs_path: logs_path.to_string(),
        artifacts_root: artifacts_root.to_string(),
        verification_path: verification_path.to_string(),
    })
}

/// Per-node inspect snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectNodeStateV1 {
    pub node_id: String,
    pub state: String,
    pub cache_decision: String,
    pub trace_path: String,
}

/// Command contract for `dag inspect`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectCommandReportV1 {
    pub run_id: String,
    pub run_state: String,
    pub nodes: Vec<InspectNodeStateV1>,
    pub artifacts: Vec<String>,
    pub failures: Vec<String>,
    pub next_action: String,
}

/// Build a run inspect report with operationally useful fields.
pub fn build_inspect_command_report(
    run_id: &str,
    run_state: &str,
    nodes: Vec<InspectNodeStateV1>,
    artifacts: Vec<String>,
    failures: Vec<String>,
    next_action: &str,
) -> Result<InspectCommandReportV1, String> {
    if run_id.trim().is_empty() {
        return Err("run_id must not be empty".to_string());
    }
    if run_state.trim().is_empty() {
        return Err("run_state must not be empty".to_string());
    }
    if nodes.is_empty() {
        return Err("inspect report must include node states".to_string());
    }
    if artifacts.is_empty() {
        return Err("inspect report must include artifacts".to_string());
    }
    if next_action.trim().is_empty() {
        return Err("next_action must not be empty".to_string());
    }
    for node in &nodes {
        for (field_name, field_value) in [
            ("node_id", node.node_id.as_str()),
            ("state", node.state.as_str()),
            ("cache_decision", node.cache_decision.as_str()),
            ("trace_path", node.trace_path.as_str()),
        ] {
            if field_value.trim().is_empty() {
                return Err(format!("inspect node {field_name} must not be empty"));
            }
        }
    }
    Ok(InspectCommandReportV1 {
        run_id: run_id.to_string(),
        run_state: run_state.to_string(),
        nodes,
        artifacts,
        failures,
        next_action: next_action.to_string(),
    })
}

/// Concise operator status view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCommandReportV1 {
    pub run_id: String,
    pub current_state: String,
    pub critical_failure: Option<String>,
    pub next_command: String,
    pub evidence_path: String,
}

/// Build concise status payload prioritizing current state and next action.
pub fn build_status_command_report(
    run_id: &str,
    current_state: &str,
    critical_failure: Option<String>,
    next_command: &str,
    evidence_path: &str,
) -> Result<StatusCommandReportV1, String> {
    for (field_name, field_value) in [
        ("run_id", run_id),
        ("current_state", current_state),
        ("next_command", next_command),
        ("evidence_path", evidence_path),
    ] {
        if field_value.trim().is_empty() {
            return Err(format!("{field_name} must not be empty"));
        }
    }
    if current_state == "failed"
        && critical_failure.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err("failed state requires critical_failure detail".to_string());
    }
    Ok(StatusCommandReportV1 {
        run_id: run_id.to_string(),
        current_state: current_state.to_string(),
        critical_failure,
        next_command: next_command.to_string(),
        evidence_path: evidence_path.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_inspect_command_report, build_plan_command_report, build_run_command_report,
        build_status_command_report, build_validate_command_report, DryPlanNodeRowV1,
        InspectNodeStateV1, PreflightCheckV1, ValidateIssueV1,
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

    #[test]
    fn g073_run_report_exposes_logs_artifacts_state_and_verification_path() {
        let report = build_run_command_report(
            "workflows/minimal.json",
            "run-20260501-001",
            "completed",
            "runs/run-20260501-001/run.log.jsonl",
            "runs/run-20260501-001/outputs",
            "runs/run-20260501-001/verify/report.json",
        )
        .expect("run report");
        assert_eq!(report.run_state, "completed");
        assert!(report.logs_path.ends_with("run.log.jsonl"));
        assert!(report.artifacts_root.ends_with("/outputs"));
        assert!(report.verification_path.ends_with("/verify/report.json"));
    }

    #[test]
    fn g074_inspect_report_includes_state_cache_trace_failures_and_next_action() {
        let report = build_inspect_command_report(
            "run-20260501-001",
            "failed",
            vec![
                InspectNodeStateV1 {
                    node_id: "align-reads".to_string(),
                    state: "completed".to_string(),
                    cache_decision: "hit".to_string(),
                    trace_path: "runs/run-20260501-001/traces/align-reads.json".to_string(),
                },
                InspectNodeStateV1 {
                    node_id: "call-variants".to_string(),
                    state: "failed".to_string(),
                    cache_decision: "miss".to_string(),
                    trace_path: "runs/run-20260501-001/traces/call-variants.json".to_string(),
                },
            ],
            vec![
                "runs/run-20260501-001/outputs/aligned.bam".to_string(),
                "runs/run-20260501-001/outputs/error.log".to_string(),
            ],
            vec!["node call-variants failed with exit code 2".to_string()],
            "run dag replay --selector failed-only",
        )
        .expect("inspect report");
        assert_eq!(report.run_state, "failed");
        assert_eq!(report.nodes.len(), 2);
        assert_eq!(report.failures.len(), 1);
        assert!(report.next_action.contains("replay"));
    }

    #[test]
    fn g075_status_report_prioritizes_state_failure_next_command_and_evidence() {
        let status = build_status_command_report(
            "run-20260501-001",
            "failed",
            Some("call-variants exited with code 2".to_string()),
            "dag inspect --run-id run-20260501-001",
            "runs/run-20260501-001/verify/report.json",
        )
        .expect("status report");
        assert_eq!(status.current_state, "failed");
        assert_eq!(
            status.critical_failure.as_deref(),
            Some("call-variants exited with code 2")
        );
        assert!(status.next_command.starts_with("dag inspect"));
        assert!(status.evidence_path.ends_with("/verify/report.json"));
    }
}
