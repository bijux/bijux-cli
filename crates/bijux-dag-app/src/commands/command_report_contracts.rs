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
    if current_state == "failed" && critical_failure.as_deref().unwrap_or("").trim().is_empty() {
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

/// Replay selector types exposed by `dag replay`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplaySelectorV1 {
    FailedOnly,
    DownstreamOf(Vec<String>),
    ChangedInputOnly,
    ForceRerun(Vec<String>),
}

/// Replay decision row for one node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayNodeDecisionV1 {
    pub node_id: String,
    pub action: String,
    pub reason: String,
}

/// Command contract for replay planning and execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCommandReportV1 {
    pub source_run_id: String,
    pub replay_run_id: String,
    pub selector: ReplaySelectorV1,
    pub decisions: Vec<ReplayNodeDecisionV1>,
    pub preserves_prior_evidence: bool,
}

/// Build replay command report with selector-driven node actions.
pub fn build_replay_command_report(
    source_run_id: &str,
    replay_run_id: &str,
    selector: ReplaySelectorV1,
    decisions: Vec<ReplayNodeDecisionV1>,
) -> Result<ReplayCommandReportV1, String> {
    for (field_name, field_value) in
        [("source_run_id", source_run_id), ("replay_run_id", replay_run_id)]
    {
        if field_value.trim().is_empty() {
            return Err(format!("{field_name} must not be empty"));
        }
    }
    if decisions.is_empty() {
        return Err("replay decisions must not be empty".to_string());
    }
    for decision in &decisions {
        for (field_name, field_value) in [
            ("node_id", decision.node_id.as_str()),
            ("action", decision.action.as_str()),
            ("reason", decision.reason.as_str()),
        ] {
            if field_value.trim().is_empty() {
                return Err(format!("replay decision {field_name} must not be empty"));
            }
        }
        if !matches!(decision.action.as_str(), "reuse" | "rerun" | "skip" | "refuse") {
            return Err(format!("invalid replay action: {}", decision.action));
        }
    }
    Ok(ReplayCommandReportV1 {
        source_run_id: source_run_id.to_string(),
        replay_run_id: replay_run_id.to_string(),
        selector,
        decisions,
        preserves_prior_evidence: true,
    })
}

/// One observed difference between two runs or bundles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffObservationV1 {
    pub surface: String,
    pub field: String,
    pub before: String,
    pub after: String,
}

/// Classification of diff meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffClassificationV1 {
    pub semantic_changes: Vec<DiffObservationV1>,
    pub noise_changes: Vec<DiffObservationV1>,
}

/// Classify diff observations into semantic versus noise.
pub fn classify_diff_observations(observations: Vec<DiffObservationV1>) -> DiffClassificationV1 {
    let mut semantic_changes = Vec::new();
    let mut noise_changes = Vec::new();
    for observation in observations {
        let is_noise = (observation.surface == "run"
            && (observation.field == "started_at" || observation.field == "finished_at"))
            || (observation.surface == "trace" && observation.field == "heartbeat_count");
        if is_noise {
            noise_changes.push(observation);
        } else {
            semantic_changes.push(observation);
        }
    }
    DiffClassificationV1 { semantic_changes, noise_changes }
}

/// Cache explain outcome class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheExplainOutcomeV1 {
    Hit,
    Miss,
    NonCacheable,
    UnsafeReuseRefused,
}

/// Direct cache explain report for one node decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheExplainReportV1 {
    pub node_id: String,
    pub outcome: CacheExplainOutcomeV1,
    pub reasons: Vec<String>,
}

/// Build direct cache explain output.
pub fn build_cache_explain_report(
    node_id: &str,
    outcome: CacheExplainOutcomeV1,
    reasons: Vec<String>,
) -> Result<CacheExplainReportV1, String> {
    if node_id.trim().is_empty() {
        return Err("node_id must not be empty".to_string());
    }
    if reasons.is_empty() {
        return Err("cache explain reasons must not be empty".to_string());
    }
    if reasons.iter().any(|reason| reason.trim().is_empty()) {
        return Err("cache explain reasons must not contain empty values".to_string());
    }
    Ok(CacheExplainReportV1 { node_id: node_id.to_string(), outcome, reasons })
}

/// Path rewrite evidence for export/import portability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundlePathRewriteV1 {
    pub original_path: String,
    pub rewritten_path: String,
}

/// Command contract for export/import portability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportImportPortabilityReportV1 {
    pub source_root: String,
    pub target_root: String,
    pub rewrites: Vec<BundlePathRewriteV1>,
    pub portable: bool,
}

/// Build portability report by rewriting source-root absolute paths to relative bundle paths.
pub fn build_export_import_portability_report(
    source_root: &str,
    target_root: &str,
    bundle_paths: Vec<String>,
) -> Result<ExportImportPortabilityReportV1, String> {
    if source_root.trim().is_empty() {
        return Err("source_root must not be empty".to_string());
    }
    if target_root.trim().is_empty() {
        return Err("target_root must not be empty".to_string());
    }
    if bundle_paths.is_empty() {
        return Err("bundle_paths must not be empty".to_string());
    }

    let source_root = source_root.trim_end_matches('/');
    let rewrites = bundle_paths
        .into_iter()
        .map(|path| {
            let rewritten = if let Some(stripped) = path.strip_prefix(source_root) {
                stripped.trim_start_matches('/').to_string()
            } else {
                path.clone()
            };
            BundlePathRewriteV1 { original_path: path, rewritten_path: rewritten }
        })
        .collect::<Vec<_>>();
    let portable = rewrites.iter().all(|entry| {
        !entry.rewritten_path.starts_with(source_root) && !entry.rewritten_path.starts_with('/')
    });
    Ok(ExportImportPortabilityReportV1 {
        source_root: source_root.to_string(),
        target_root: target_root.to_string(),
        rewrites,
        portable,
    })
}

/// Doctor finding with one explicit remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorFindingV1 {
    pub finding_id: String,
    pub severity: String,
    pub message: String,
    pub remediation: String,
}

/// Command contract for `dag doctor`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorCommandReportV1 {
    pub status: String,
    pub findings: Vec<DoctorFindingV1>,
}

/// Build doctor report with exactly one remediation per finding.
pub fn build_doctor_command_report(
    status: &str,
    findings: Vec<DoctorFindingV1>,
) -> Result<DoctorCommandReportV1, String> {
    if status.trim().is_empty() {
        return Err("status must not be empty".to_string());
    }
    for finding in &findings {
        for (field_name, field_value) in [
            ("finding_id", finding.finding_id.as_str()),
            ("severity", finding.severity.as_str()),
            ("message", finding.message.as_str()),
            ("remediation", finding.remediation.as_str()),
        ] {
            if field_value.trim().is_empty() {
                return Err(format!("doctor finding {field_name} must not be empty"));
            }
        }
    }
    Ok(DoctorCommandReportV1 { status: status.to_string(), findings })
}

#[cfg(test)]
mod tests {
    use super::{
        build_cache_explain_report, build_doctor_command_report,
        build_export_import_portability_report, build_inspect_command_report,
        build_plan_command_report, build_replay_command_report, build_run_command_report,
        build_status_command_report, build_validate_command_report, classify_diff_observations,
        CacheExplainOutcomeV1, DiffObservationV1, DoctorFindingV1, DryPlanNodeRowV1,
        InspectNodeStateV1, PreflightCheckV1, ReplayNodeDecisionV1, ReplaySelectorV1,
        ValidateIssueV1,
    };

    #[test]
    fn validate_report_makes_invalid_graphs_fixable_from_output() {
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
        assert_eq!(report.hard_failures[0].remediation, "set output path under declared run root");
        assert_eq!(report.lint_findings[0].severity, "warn");
    }

    #[test]
    fn plan_report_exposes_fingerprint_artifacts_cache_and_preflight() {
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
    fn run_report_exposes_logs_artifacts_state_and_verification_path() {
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
    fn inspect_report_includes_state_cache_trace_failures_and_next_action() {
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
    fn status_report_prioritizes_state_failure_next_command_and_evidence() {
        let status = build_status_command_report(
            "run-20260501-001",
            "failed",
            Some("call-variants exited with code 2".to_string()),
            "dag inspect --run-id run-20260501-001",
            "runs/run-20260501-001/verify/report.json",
        )
        .expect("status report");
        assert_eq!(status.current_state, "failed");
        assert_eq!(status.critical_failure.as_deref(), Some("call-variants exited with code 2"));
        assert!(status.next_command.starts_with("dag inspect"));
        assert!(status.evidence_path.ends_with("/verify/report.json"));
    }

    #[test]
    fn replay_report_supports_selectors_and_preserves_prior_evidence() {
        let report = build_replay_command_report(
            "run-20260501-001",
            "run-20260501-002",
            ReplaySelectorV1::FailedOnly,
            vec![
                ReplayNodeDecisionV1 {
                    node_id: "align-reads".to_string(),
                    action: "reuse".to_string(),
                    reason: "upstream outputs unchanged".to_string(),
                },
                ReplayNodeDecisionV1 {
                    node_id: "call-variants".to_string(),
                    action: "rerun".to_string(),
                    reason: "node failed in source run".to_string(),
                },
            ],
        )
        .expect("replay report");
        assert_eq!(report.source_run_id, "run-20260501-001");
        assert_eq!(report.replay_run_id, "run-20260501-002");
        assert!(report.preserves_prior_evidence);
        assert_eq!(report.decisions.len(), 2);
    }

    #[test]
    fn diff_classification_separates_semantic_change_from_noise() {
        let classified = classify_diff_observations(vec![
            DiffObservationV1 {
                surface: "graph".to_string(),
                field: "node_count".to_string(),
                before: "12".to_string(),
                after: "13".to_string(),
            },
            DiffObservationV1 {
                surface: "run".to_string(),
                field: "started_at".to_string(),
                before: "2026-05-01T09:00:00Z".to_string(),
                after: "2026-05-01T09:01:00Z".to_string(),
            },
            DiffObservationV1 {
                surface: "artifact".to_string(),
                field: "content_hash".to_string(),
                before: "a1".to_string(),
                after: "a2".to_string(),
            },
        ]);
        assert_eq!(classified.semantic_changes.len(), 2);
        assert_eq!(classified.noise_changes.len(), 1);
        assert_eq!(classified.noise_changes[0].field, "started_at");
    }

    #[test]
    fn cache_explain_is_direct_for_hit_miss_noncacheable_and_refusal() {
        let hit = build_cache_explain_report(
            "align-reads",
            CacheExplainOutcomeV1::Hit,
            vec!["cache key matched all compatibility factors".to_string()],
        )
        .expect("cache hit explain");
        let miss = build_cache_explain_report(
            "call-variants",
            CacheExplainOutcomeV1::Miss,
            vec!["input fingerprint changed".to_string()],
        )
        .expect("cache miss explain");
        let non_cacheable = build_cache_explain_report(
            "collect-clock",
            CacheExplainOutcomeV1::NonCacheable,
            vec!["node declared nondeterministic side effects".to_string()],
        )
        .expect("non-cacheable explain");
        let refusal = build_cache_explain_report(
            "publish",
            CacheExplainOutcomeV1::UnsafeReuseRefused,
            vec!["integrity verification failed for cached artifact".to_string()],
        )
        .expect("unsafe refusal explain");

        assert!(matches!(hit.outcome, CacheExplainOutcomeV1::Hit));
        assert!(matches!(miss.outcome, CacheExplainOutcomeV1::Miss));
        assert!(matches!(non_cacheable.outcome, CacheExplainOutcomeV1::NonCacheable));
        assert!(matches!(refusal.outcome, CacheExplainOutcomeV1::UnsafeReuseRefused));
    }

    #[test]
    fn export_import_report_rewrites_paths_for_portability() {
        let report = build_export_import_portability_report(
            "/workspace/runs/run-1",
            "/clean-room/imported",
            vec![
                "/workspace/runs/run-1/manifest.json".to_string(),
                "/workspace/runs/run-1/outputs/variants.vcf".to_string(),
                "relative/proof.json".to_string(),
            ],
        )
        .expect("portability report");
        assert!(report.portable);
        assert_eq!(report.rewrites[0].rewritten_path, "manifest.json");
        assert_eq!(report.rewrites[1].rewritten_path, "outputs/variants.vcf");
        assert_eq!(report.rewrites[2].rewritten_path, "relative/proof.json");
    }

    #[test]
    fn doctor_report_provides_one_remediation_per_finding() {
        let report = build_doctor_command_report(
            "degraded",
            vec![
                DoctorFindingV1 {
                    finding_id: "doctor.run_root_unwritable".to_string(),
                    severity: "error".to_string(),
                    message: "run root is not writable".to_string(),
                    remediation: "grant write permission or choose --root under writable path"
                        .to_string(),
                },
                DoctorFindingV1 {
                    finding_id: "doctor.adapter_shell_missing".to_string(),
                    severity: "warn".to_string(),
                    message: "/bin/sh not found in configured execution image".to_string(),
                    remediation:
                        "install shell adapter runtime or switch workflow to const adapter"
                            .to_string(),
                },
            ],
        )
        .expect("doctor report");
        assert_eq!(report.status, "degraded");
        assert_eq!(report.findings.len(), 2);
        for finding in &report.findings {
            assert!(!finding.remediation.trim().is_empty());
        }
    }
}
