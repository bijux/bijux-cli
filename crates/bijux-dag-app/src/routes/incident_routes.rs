use crate::commands::{DagCli, IncidentCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    evaluate_slo, health_dashboard_score, integrated_verification_lane_default,
    release_policy_allows, replay_trust_warnings, AuditReadinessChecklist, ErrorBudgetPolicy,
    GamedayScenario, IncidentClassification, IncidentSeverity, IntegratedVerificationLane,
    LifecycleGovernanceRule, OperatorTrainingCatalog, PlatformAcceptanceBoard,
    PlatformHealthDashboard, PlatformInvariantCatalog, PostmortemTemplate, ProductBoundary,
    ReleaseGovernancePolicy, RunProvenanceAttestation, RunbookEntry, ServiceLevelIndicators,
    ServiceLevelObjective, SupportabilityModel, SustainabilityOwnership,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct IncidentModeSimulation {
    classification: IncidentClassification,
    dashboard: PlatformHealthDashboard,
    #[serde(default)]
    disabled_actions: Vec<String>,
    #[serde(default)]
    elevated_surfaces: Vec<String>,
    #[serde(default)]
    retained_actions: Vec<String>,
    #[serde(default)]
    runbooks: Vec<RunbookEntry>,
    supportability: SupportabilityModel,
}

#[derive(Debug, Serialize)]
struct IncidentModeReport {
    incident_type: String,
    severity: String,
    dashboard_score: f64,
    disabled_actions: Vec<String>,
    elevated_surfaces: Vec<String>,
    retained_actions: Vec<String>,
    required_evidence: Vec<String>,
    supported_backends: Vec<String>,
    official_plugins: Vec<String>,
    gaps: Vec<String>,
    incident_mode_ready: bool,
}

#[derive(Debug, Deserialize)]
struct BlastRadiusSimulation {
    classification: IncidentClassification,
    failing_components: Vec<String>,
    #[serde(default)]
    workflows: Vec<WorkflowImpact>,
    #[serde(default)]
    service_context: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WorkflowImpact {
    workflow_id: String,
    tenant_id: String,
    #[serde(default)]
    artifact_ids: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BlastRadiusReport {
    incident_type: String,
    severity: String,
    failing_components: Vec<String>,
    impacted_workflows: Vec<String>,
    impacted_tenants: Vec<String>,
    impacted_artifacts: Vec<String>,
    impacted_services: BTreeMap<String, Vec<String>>,
    gaps: Vec<String>,
    blast_radius_ready: bool,
}

#[derive(Debug, Deserialize)]
struct SafeStopSimulation {
    incident_id: String,
    #[serde(default)]
    stop_scope: Vec<String>,
    queued_runs: usize,
    running_runs: usize,
    stop_new_schedules: bool,
    stop_new_dispatch: bool,
    drain_running_work: bool,
    preserve_artifact_commits: bool,
    #[serde(default)]
    restart_conditions: Vec<String>,
    runbook: RunbookEntry,
    approval_required: bool,
}

#[derive(Debug, Serialize)]
struct SafeStopReport {
    incident_id: String,
    stop_scope: Vec<String>,
    queued_runs: usize,
    running_runs: usize,
    restart_conditions: Vec<String>,
    approval_required: bool,
    required_evidence: Vec<String>,
    gaps: Vec<String>,
    safe_stop_ready: bool,
}

#[derive(Debug, Deserialize)]
struct DegradedModeSimulation {
    #[serde(default)]
    missing_dependencies: Vec<String>,
    read_only_mode: bool,
    limited_submit_mode: bool,
    queue_drain_mode: bool,
    #[serde(default)]
    blocked_actions: Vec<String>,
    #[serde(default)]
    available_actions: Vec<String>,
    boundary: ProductBoundary,
}

#[derive(Debug, Serialize)]
struct DegradedModeReport {
    missing_dependencies: Vec<String>,
    available_modes: Vec<String>,
    blocked_actions: Vec<String>,
    available_actions: Vec<String>,
    platform_guarantees: Vec<String>,
    operator_responsibilities: Vec<String>,
    gaps: Vec<String>,
    degraded_mode_ready: bool,
}

#[derive(Debug, Deserialize)]
struct IncidentAnnotationSimulation {
    incident_id: String,
    author: String,
    unix_ms: u128,
    note: String,
    #[serde(default)]
    run_ids: Vec<String>,
    #[serde(default)]
    tenant_ids: Vec<String>,
    #[serde(default)]
    artifact_ids: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    searchable_fields: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct IncidentAnnotationReport {
    incident_id: String,
    author: String,
    unix_ms: u128,
    run_ids: Vec<String>,
    tenant_ids: Vec<String>,
    artifact_ids: Vec<String>,
    tags: Vec<String>,
    search_index_keys: Vec<String>,
    gaps: Vec<String>,
    annotation_ready: bool,
}

#[derive(Debug, Deserialize)]
struct RepairWindowSimulation {
    incident_id: String,
    requested_by: String,
    #[serde(default)]
    approved_by: Vec<String>,
    start_utc: String,
    end_utc: String,
    #[serde(default)]
    scope: Vec<String>,
    #[serde(default)]
    run_ids: Vec<String>,
    #[serde(default)]
    tenant_ids: Vec<String>,
    #[serde(default)]
    repair_actions: Vec<String>,
    #[serde(default)]
    outcome_tracking: Vec<String>,
    lifecycle_rule: LifecycleGovernanceRule,
}

#[derive(Debug, Serialize)]
struct RepairWindowReport {
    incident_id: String,
    requested_by: String,
    approved_by: Vec<String>,
    start_utc: String,
    end_utc: String,
    scope: Vec<String>,
    run_ids: Vec<String>,
    tenant_ids: Vec<String>,
    repair_actions: Vec<String>,
    outcome_tracking: Vec<String>,
    lifecycle_state: String,
    gaps: Vec<String>,
    repair_window_ready: bool,
}

#[derive(Debug, Deserialize)]
struct IncidentTimelineSimulation {
    incident_id: String,
    events: Vec<IncidentTimelineEvent>,
    postmortem: PostmortemTemplate,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct IncidentTimelineEvent {
    unix_ms: u128,
    source: String,
    action: String,
    phase: String,
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    tenant_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct IncidentTimelineReport {
    incident_id: String,
    phases_present: Vec<String>,
    event_count: usize,
    ordered_events: Vec<IncidentTimelineEvent>,
    missing_postmortem_sections: Vec<String>,
    gaps: Vec<String>,
    timeline_ready: bool,
}

#[derive(Debug, Deserialize)]
struct ReplayValidationSimulation {
    incident_id: String,
    baseline: RunProvenanceAttestation,
    candidate: RunProvenanceAttestation,
    release_policy: ReleaseGovernancePolicy,
    outputs_match: bool,
    lineage_consistent: bool,
}

#[derive(Debug, Serialize)]
struct ReplayValidationReport {
    incident_id: String,
    replay_trust_warnings: Vec<String>,
    release_policy_ready: bool,
    outputs_match: bool,
    lineage_consistent: bool,
    gaps: Vec<String>,
    replay_validation_ready: bool,
}

#[derive(Debug, Deserialize)]
struct ReadinessReviewSimulation {
    workflow_id: String,
    #[serde(default)]
    owners: Vec<String>,
    #[serde(default)]
    runbooks: Vec<RunbookEntry>,
    training: OperatorTrainingCatalog,
    audit_checklist: AuditReadinessChecklist,
    objective: ServiceLevelObjective,
    indicators: ServiceLevelIndicators,
    #[serde(default)]
    verification_lane: Option<IntegratedVerificationLane>,
    supportability: SupportabilityModel,
}

#[derive(Debug, Serialize)]
struct ReadinessReviewReport {
    workflow_id: String,
    owners: Vec<String>,
    runbook_count: usize,
    training_interventions: Vec<String>,
    audit_checks: Vec<String>,
    required_verification_domains: Vec<String>,
    slo_violations: Vec<String>,
    supported_backends: Vec<String>,
    gaps: Vec<String>,
    readiness_review_passed: bool,
}

#[derive(Debug, Deserialize)]
struct ResilienceScorecardSimulation {
    workflow_id: String,
    objective: ServiceLevelObjective,
    indicators: ServiceLevelIndicators,
    dashboard: PlatformHealthDashboard,
    #[serde(default)]
    gamedays: Vec<GamedayScenario>,
    postmortem: PostmortemTemplate,
    #[serde(default)]
    invariants: Option<PlatformInvariantCatalog>,
    #[serde(default)]
    verification_lane: Option<IntegratedVerificationLane>,
    supportability: SupportabilityModel,
    sustainability: SustainabilityOwnership,
    acceptance_board: PlatformAcceptanceBoard,
    error_budget: ErrorBudgetPolicy,
}

#[derive(Debug, Serialize)]
struct ResilienceScorecardReport {
    workflow_id: String,
    score: f64,
    slo_passed: bool,
    dashboard_score: f64,
    gameday_coverage: usize,
    invariant_count: usize,
    supported_backend_count: usize,
    subsystem_owner_count: usize,
    acceptance_criteria_count: usize,
    weaknesses: Vec<String>,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn severity_name(severity: &IncidentSeverity) -> &'static str {
    match severity {
        IncidentSeverity::Critical => "critical",
        IncidentSeverity::High => "high",
        IncidentSeverity::Medium => "medium",
        IncidentSeverity::Low => "low",
    }
}

fn incident_mode_payload(simulation: IncidentModeSimulation) -> (serde_json::Value, bool) {
    let IncidentModeSimulation {
        classification,
        dashboard,
        disabled_actions,
        elevated_surfaces,
        retained_actions,
        runbooks,
        supportability,
    } = simulation;
    let dashboard_score = health_dashboard_score(&dashboard);
    let mut gaps = Vec::new();
    if disabled_actions.is_empty() {
        gaps.push("incident mode should suppress at least one nonessential action".to_string());
    }
    if elevated_surfaces.is_empty() {
        gaps.push("incident mode should elevate critical visibility surfaces".to_string());
    }
    if retained_actions.is_empty() {
        gaps.push("incident mode should preserve a minimal safe operator action set".to_string());
    }
    if runbooks.is_empty() {
        gaps.push("incident mode should point to at least one runbook".to_string());
    }
    if supportability.supported_backends.is_empty() {
        gaps.push("incident mode should name supported execution backends".to_string());
    }
    if dashboard_score < 0.60 {
        gaps.push("dashboard health is too weak for confident incident operation".to_string());
    }
    let required_evidence = runbooks
        .iter()
        .flat_map(|entry| entry.required_evidence.iter().cloned())
        .collect::<Vec<_>>();
    let supported_backends = supportability.supported_backends.into_iter().collect::<Vec<_>>();
    let official_plugins = supportability.official_plugins.into_iter().collect::<Vec<_>>();
    let report = IncidentModeReport {
        incident_type: classification.incident_type,
        severity: severity_name(&classification.severity).to_string(),
        dashboard_score,
        disabled_actions,
        elevated_surfaces,
        retained_actions,
        required_evidence,
        supported_backends,
        official_plugins,
        incident_mode_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.incident_mode_ready;
    (serde_json::to_value(report).expect("incident mode report"), ok)
}

fn blast_radius_payload(simulation: BlastRadiusSimulation) -> (serde_json::Value, bool) {
    let BlastRadiusSimulation { classification, failing_components, workflows, service_context } =
        simulation;
    let failing_set = failing_components.iter().cloned().collect::<BTreeSet<_>>();
    let impacted_workflows = workflows
        .iter()
        .filter(|workflow| {
            workflow.dependencies.iter().any(|dependency| failing_set.contains(dependency))
        })
        .collect::<Vec<_>>();
    let impacted_workflow_ids =
        impacted_workflows.iter().map(|workflow| workflow.workflow_id.clone()).collect::<Vec<_>>();
    let impacted_tenants = impacted_workflows
        .iter()
        .map(|workflow| workflow.tenant_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let impacted_artifacts = impacted_workflows
        .iter()
        .flat_map(|workflow| workflow.artifact_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let impacted_services = service_context
        .into_iter()
        .filter(|(component, _)| failing_set.contains(component))
        .collect::<BTreeMap<_, _>>();
    let mut gaps = Vec::new();
    if failing_components.is_empty() {
        gaps.push("blast-radius analysis requires at least one failing component".to_string());
    }
    if impacted_workflow_ids.is_empty() {
        gaps.push("blast-radius analysis did not resolve any impacted workflows".to_string());
    }
    if impacted_tenants.is_empty() {
        gaps.push("blast-radius analysis should resolve at least one impacted tenant".to_string());
    }
    if impacted_services.len() != failing_set.len() {
        gaps.push("service context is incomplete for one or more failing components".to_string());
    }
    let report = BlastRadiusReport {
        incident_type: classification.incident_type,
        severity: severity_name(&classification.severity).to_string(),
        failing_components,
        impacted_workflows: impacted_workflow_ids,
        impacted_tenants,
        impacted_artifacts,
        impacted_services,
        blast_radius_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.blast_radius_ready;
    (serde_json::to_value(report).expect("blast radius report"), ok)
}

fn safe_stop_payload(simulation: SafeStopSimulation) -> (serde_json::Value, bool) {
    let SafeStopSimulation {
        incident_id,
        stop_scope,
        queued_runs,
        running_runs,
        stop_new_schedules,
        stop_new_dispatch,
        drain_running_work,
        preserve_artifact_commits,
        restart_conditions,
        runbook,
        approval_required,
    } = simulation;
    let mut gaps = Vec::new();
    if stop_scope.is_empty() {
        gaps.push("safe-stop requires an explicit stop scope".to_string());
    }
    if !stop_new_schedules {
        gaps.push("safe-stop must halt new schedule creation".to_string());
    }
    if !stop_new_dispatch {
        gaps.push("safe-stop must halt new task dispatch".to_string());
    }
    if !drain_running_work {
        gaps.push(
            "safe-stop must define whether running work drains or is interrupted".to_string(),
        );
    }
    if !preserve_artifact_commits {
        gaps.push("safe-stop should preserve artifact commit integrity".to_string());
    }
    if restart_conditions.is_empty() {
        gaps.push("safe-stop requires explicit restart conditions".to_string());
    }
    if runbook.required_evidence.is_empty() {
        gaps.push("safe-stop runbook must require recovery evidence".to_string());
    }
    let report = SafeStopReport {
        incident_id,
        stop_scope,
        queued_runs,
        running_runs,
        restart_conditions,
        approval_required,
        required_evidence: runbook.required_evidence,
        safe_stop_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.safe_stop_ready;
    (serde_json::to_value(report).expect("safe stop report"), ok)
}

fn degraded_mode_payload(simulation: DegradedModeSimulation) -> (serde_json::Value, bool) {
    let DegradedModeSimulation {
        missing_dependencies,
        read_only_mode,
        limited_submit_mode,
        queue_drain_mode,
        blocked_actions,
        available_actions,
        boundary,
    } = simulation;
    let mut available_modes = Vec::new();
    if read_only_mode {
        available_modes.push("read-only".to_string());
    }
    if limited_submit_mode {
        available_modes.push("limited-submit".to_string());
    }
    if queue_drain_mode {
        available_modes.push("queue-drain".to_string());
    }
    let mut gaps = Vec::new();
    if missing_dependencies.is_empty() {
        gaps.push("degraded mode should state which dependencies are unavailable".to_string());
    }
    if available_modes.is_empty() {
        gaps.push("degraded mode should preserve at least one useful operating mode".to_string());
    }
    if blocked_actions.is_empty() {
        gaps.push("degraded mode should make blocked actions explicit".to_string());
    }
    if available_actions.is_empty() {
        gaps.push("degraded mode should preserve at least one safe operator action".to_string());
    }
    if boundary.platform_guarantees.is_empty() {
        gaps.push("degraded mode should state which platform guarantees still hold".to_string());
    }
    if boundary.operator_responsibilities.is_empty() {
        gaps.push(
            "degraded mode should state the operator responsibilities that remain".to_string(),
        );
    }
    let report = DegradedModeReport {
        missing_dependencies,
        available_modes,
        blocked_actions,
        available_actions,
        platform_guarantees: boundary.platform_guarantees,
        operator_responsibilities: boundary.operator_responsibilities,
        degraded_mode_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.degraded_mode_ready;
    (serde_json::to_value(report).expect("degraded mode report"), ok)
}

fn annotation_payload(simulation: IncidentAnnotationSimulation) -> (serde_json::Value, bool) {
    let IncidentAnnotationSimulation {
        incident_id,
        author,
        unix_ms,
        note,
        run_ids,
        tenant_ids,
        artifact_ids,
        tags,
        searchable_fields,
    } = simulation;
    let mut gaps = Vec::new();
    if note.trim().is_empty() {
        gaps.push("incident annotation requires a non-empty note".to_string());
    }
    if run_ids.is_empty() && tenant_ids.is_empty() && artifact_ids.is_empty() {
        gaps.push(
            "incident annotation should link at least one run, tenant, or artifact".to_string(),
        );
    }
    if author.trim().is_empty() {
        gaps.push("incident annotation requires an author".to_string());
    }
    if searchable_fields.get("incident_id").map(String::as_str) != Some(incident_id.as_str()) {
        gaps.push("incident annotation should index incident_id in searchable fields".to_string());
    }
    if searchable_fields.get("author").map(String::as_str) != Some(author.as_str()) {
        gaps.push("incident annotation should index author in searchable fields".to_string());
    }
    let report = IncidentAnnotationReport {
        incident_id,
        author,
        unix_ms,
        run_ids,
        tenant_ids,
        artifact_ids,
        tags,
        search_index_keys: searchable_fields.keys().cloned().collect::<Vec<_>>(),
        annotation_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.annotation_ready;
    (serde_json::to_value(report).expect("incident annotation report"), ok)
}

fn repair_window_payload(simulation: RepairWindowSimulation) -> (serde_json::Value, bool) {
    let RepairWindowSimulation {
        incident_id,
        requested_by,
        approved_by,
        start_utc,
        end_utc,
        scope,
        run_ids,
        tenant_ids,
        repair_actions,
        outcome_tracking,
        lifecycle_rule,
    } = simulation;
    let mut gaps = Vec::new();
    if requested_by.trim().is_empty() {
        gaps.push("repair window requires a requesting operator".to_string());
    }
    if approved_by.is_empty() {
        gaps.push("repair window requires at least one approval".to_string());
    }
    if start_utc.trim().is_empty() || end_utc.trim().is_empty() || start_utc == end_utc {
        gaps.push("repair window requires distinct start and end timestamps".to_string());
    }
    if scope.is_empty() {
        gaps.push("repair window requires explicit repair scope".to_string());
    }
    if run_ids.is_empty() && tenant_ids.is_empty() {
        gaps.push("repair window should target at least one run or tenant".to_string());
    }
    if repair_actions.is_empty() {
        gaps.push("repair window requires at least one repair action".to_string());
    }
    if outcome_tracking.is_empty() {
        gaps.push("repair window requires outcome tracking fields".to_string());
    }
    if lifecycle_rule.state.trim().is_empty() {
        gaps.push("repair window requires a lifecycle governance state".to_string());
    }
    let report = RepairWindowReport {
        incident_id,
        requested_by,
        approved_by,
        start_utc,
        end_utc,
        scope,
        run_ids,
        tenant_ids,
        repair_actions,
        outcome_tracking,
        lifecycle_state: lifecycle_rule.state,
        repair_window_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.repair_window_ready;
    (serde_json::to_value(report).expect("repair window report"), ok)
}

fn timeline_payload(simulation: IncidentTimelineSimulation) -> (serde_json::Value, bool) {
    let IncidentTimelineSimulation { incident_id, mut events, postmortem } = simulation;
    events.sort_by_key(|event| event.unix_ms);
    let phases_present = events
        .iter()
        .map(|event| event.phase.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let required_postmortem_sections = ["summary", "impact", "timeline", "actions"];
    let available_sections = postmortem
        .required_sections
        .iter()
        .map(|section| section.trim().to_lowercase())
        .collect::<BTreeSet<_>>();
    let missing_postmortem_sections = required_postmortem_sections
        .iter()
        .filter(|section| !available_sections.contains(**section))
        .map(|section| (*section).to_string())
        .collect::<Vec<_>>();
    let mut gaps = Vec::new();
    if events.is_empty() {
        gaps.push("incident timeline requires at least one event".to_string());
    }
    for phase in ["detection", "mitigation", "recovery"] {
        if !phases_present.iter().any(|present| present == phase) {
            gaps.push(format!("incident timeline is missing the {phase} phase"));
        }
    }
    if !missing_postmortem_sections.is_empty() {
        gaps.push("postmortem template is missing required incident sections".to_string());
    }
    let report = IncidentTimelineReport {
        incident_id,
        phases_present,
        event_count: events.len(),
        ordered_events: events,
        missing_postmortem_sections,
        timeline_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.timeline_ready;
    (serde_json::to_value(report).expect("incident timeline report"), ok)
}

fn replay_validation_payload(simulation: ReplayValidationSimulation) -> (serde_json::Value, bool) {
    let ReplayValidationSimulation {
        incident_id,
        baseline,
        candidate,
        release_policy,
        outputs_match,
        lineage_consistent,
    } = simulation;
    let trust = replay_trust_warnings(&candidate.run_id, &baseline, &candidate);
    let release_policy_ready = release_policy_allows(&release_policy);
    let mut gaps = Vec::new();
    if !trust.warnings.is_empty() {
        gaps.push("replay validation detected provenance drift".to_string());
    }
    if !release_policy_ready {
        gaps.push(
            "replay validation requires evidence, compatibility results, and rollback plan"
                .to_string(),
        );
    }
    if !outputs_match {
        gaps.push("replayed outputs do not match the expected recovery result".to_string());
    }
    if !lineage_consistent {
        gaps.push("replayed lineage is not consistent with the baseline run".to_string());
    }
    let report = ReplayValidationReport {
        incident_id,
        replay_trust_warnings: trust.warnings,
        release_policy_ready,
        outputs_match,
        lineage_consistent,
        replay_validation_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.replay_validation_ready;
    (serde_json::to_value(report).expect("replay validation report"), ok)
}

fn readiness_review_payload(simulation: ReadinessReviewSimulation) -> (serde_json::Value, bool) {
    let ReadinessReviewSimulation {
        workflow_id,
        owners,
        runbooks,
        training,
        audit_checklist,
        objective,
        indicators,
        verification_lane,
        supportability,
    } = simulation;
    let slo = evaluate_slo(&objective, &indicators);
    let verification_lane = verification_lane.unwrap_or_else(integrated_verification_lane_default);
    let mut gaps = Vec::new();
    if owners.is_empty() {
        gaps.push("readiness review requires explicit owners".to_string());
    }
    if runbooks.is_empty() {
        gaps.push("readiness review requires at least one runbook".to_string());
    }
    if training.interventions.is_empty() {
        gaps.push("readiness review requires operator training interventions".to_string());
    }
    if audit_checklist.checks.is_empty() {
        gaps.push("readiness review requires audit checks".to_string());
    }
    if supportability.supported_backends.is_empty() {
        gaps.push("readiness review requires supported backend declarations".to_string());
    }
    if !slo.passed {
        gaps.push("readiness review requires SLO conformance".to_string());
    }
    let report = ReadinessReviewReport {
        workflow_id,
        owners,
        runbook_count: runbooks.len(),
        training_interventions: training.interventions,
        audit_checks: audit_checklist.checks,
        required_verification_domains: verification_lane.required_domains,
        slo_violations: slo.violations,
        supported_backends: supportability.supported_backends.into_iter().collect::<Vec<_>>(),
        readiness_review_passed: gaps.is_empty(),
        gaps,
    };
    let ok = report.readiness_review_passed;
    (serde_json::to_value(report).expect("readiness review report"), ok)
}

fn scorecard_payload(simulation: ResilienceScorecardSimulation) -> (serde_json::Value, bool) {
    let ResilienceScorecardSimulation {
        workflow_id,
        objective,
        indicators,
        dashboard,
        gamedays,
        postmortem,
        invariants,
        verification_lane,
        supportability,
        sustainability,
        acceptance_board,
        error_budget,
    } = simulation;
    let slo = evaluate_slo(&objective, &indicators);
    let dashboard_score = health_dashboard_score(&dashboard);
    let invariant_catalog = invariants.unwrap_or_else(|| PlatformInvariantCatalog {
        invariants: vec![
            "deterministic planning".to_string(),
            "immutable artifact identity".to_string(),
            "tenant isolation cannot be bypassed".to_string(),
        ],
    });
    let verification_lane = verification_lane.unwrap_or_else(integrated_verification_lane_default);
    let mut score = 0.0_f64;
    let mut weaknesses = Vec::new();
    if slo.passed {
        score += 25.0;
    } else {
        weaknesses.push("SLO commitments are not currently met".to_string());
    }
    score += dashboard_score * 20.0;
    if gamedays.is_empty() {
        weaknesses.push("no gameday drills are defined".to_string());
    } else {
        score += 10.0;
    }
    let required_sections = ["summary", "impact", "timeline", "actions"];
    let postmortem_sections = postmortem
        .required_sections
        .iter()
        .map(|section| section.trim().to_lowercase())
        .collect::<BTreeSet<_>>();
    let missing_sections =
        required_sections.iter().filter(|section| !postmortem_sections.contains(**section)).count();
    if missing_sections == 0 {
        score += 10.0;
    } else {
        weaknesses.push("postmortem template is missing core sections".to_string());
    }
    if !invariant_catalog.invariants.is_empty() {
        score += 10.0;
    } else {
        weaknesses.push("no platform invariants are recorded".to_string());
    }
    if !supportability.supported_backends.is_empty() {
        score += 10.0;
    } else {
        weaknesses.push("no supported backend set is declared".to_string());
    }
    if !sustainability.subsystem_owners.is_empty() && !sustainability.review_routing.is_empty() {
        score += 10.0;
    } else {
        weaknesses.push("sustainability ownership is incomplete".to_string());
    }
    if !acceptance_board.preview_to_stable_criteria.is_empty() {
        score += 5.0;
    } else {
        weaknesses.push("acceptance board criteria are missing".to_string());
    }
    if !verification_lane.required_evidence.is_empty() {
        score += 5.0;
    } else {
        weaknesses.push("verification lane evidence is incomplete".to_string());
    }
    if error_budget.scheduler_outage_minutes_per_quarter <= 0.0
        || error_budget.backend_degradation_minutes_per_quarter <= 0.0
    {
        weaknesses.push("error budget policy is not defined tightly enough".to_string());
    } else {
        score += 5.0;
    }
    let score = score.min(100.0);
    let report = ResilienceScorecardReport {
        workflow_id,
        score,
        slo_passed: slo.passed,
        dashboard_score,
        gameday_coverage: gamedays.len(),
        invariant_count: invariant_catalog.invariants.len(),
        supported_backend_count: supportability.supported_backends.len(),
        subsystem_owner_count: sustainability.subsystem_owners.len(),
        acceptance_criteria_count: acceptance_board.preview_to_stable_criteria.len(),
        weaknesses,
    };
    let ok = report.score >= 75.0 && report.weaknesses.is_empty();
    (serde_json::to_value(report).expect("resilience scorecard report"), ok)
}

pub(crate) fn handle_incident_command(
    cli: &DagCli,
    command: &IncidentCommands,
) -> Result<ExitCode, ExitCode> {
    let (surface, payload, ok) = match command {
        IncidentCommands::Mode { simulation } => {
            let simulation: IncidentModeSimulation = parse_json_file(simulation)?;
            let (payload, ok) = incident_mode_payload(simulation);
            ("dag.incident.mode", payload, ok)
        }
        IncidentCommands::BlastRadius { simulation } => {
            let simulation: BlastRadiusSimulation = parse_json_file(simulation)?;
            let (payload, ok) = blast_radius_payload(simulation);
            ("dag.incident.blast-radius", payload, ok)
        }
        IncidentCommands::SafeStop { simulation } => {
            let simulation: SafeStopSimulation = parse_json_file(simulation)?;
            let (payload, ok) = safe_stop_payload(simulation);
            ("dag.incident.safe-stop", payload, ok)
        }
        IncidentCommands::DegradedMode { simulation } => {
            let simulation: DegradedModeSimulation = parse_json_file(simulation)?;
            let (payload, ok) = degraded_mode_payload(simulation);
            ("dag.incident.degraded-mode", payload, ok)
        }
        IncidentCommands::Annotation { simulation } => {
            let simulation: IncidentAnnotationSimulation = parse_json_file(simulation)?;
            let (payload, ok) = annotation_payload(simulation);
            ("dag.incident.annotation", payload, ok)
        }
        IncidentCommands::RepairWindow { simulation } => {
            let simulation: RepairWindowSimulation = parse_json_file(simulation)?;
            let (payload, ok) = repair_window_payload(simulation);
            ("dag.incident.repair-window", payload, ok)
        }
        IncidentCommands::Timeline { simulation } => {
            let simulation: IncidentTimelineSimulation = parse_json_file(simulation)?;
            let (payload, ok) = timeline_payload(simulation);
            ("dag.incident.timeline", payload, ok)
        }
        IncidentCommands::ReplayValidation { simulation } => {
            let simulation: ReplayValidationSimulation = parse_json_file(simulation)?;
            let (payload, ok) = replay_validation_payload(simulation);
            ("dag.incident.replay-validation", payload, ok)
        }
        IncidentCommands::ReadinessReview { simulation } => {
            let simulation: ReadinessReviewSimulation = parse_json_file(simulation)?;
            let (payload, ok) = readiness_review_payload(simulation);
            ("dag.incident.readiness-review", payload, ok)
        }
        IncidentCommands::Scorecard { simulation } => {
            let simulation: ResilienceScorecardSimulation = parse_json_file(simulation)?;
            let (payload, ok) = scorecard_payload(simulation);
            ("dag.incident.scorecard", payload, ok)
        }
    };
    emit_json(
        cli,
        surface,
        ok,
        payload,
        if ok {
            Vec::new()
        } else {
            vec![
                json!({"message":"incident posture is incomplete","remediation":"fill the reported incident-mode gaps before treating the workflow family as incident-ready"}),
            ]
        },
        if ok { ExitCode::SUCCESS } else { ExitCode::from(2) },
    )
}

#[cfg(test)]
mod tests {
    use super::{
        annotation_payload, blast_radius_payload, degraded_mode_payload, incident_mode_payload,
        readiness_review_payload, repair_window_payload, replay_validation_payload,
        safe_stop_payload, scorecard_payload, timeline_payload,
    };
    use bijux_dag_runtime::simulated_platform::{
        AuditReadinessChecklist, BinaryComponent, BinaryProvenanceRecord, EnvironmentAttestation,
        ErrorBudgetPolicy, GamedayScenario, IncidentClassification, IncidentSeverity,
        IntegratedVerificationLane, LifecycleGovernanceRule, OperatorTrainingCatalog,
        PlatformAcceptanceBoard, PlatformHealthDashboard, PlatformInvariantCatalog,
        PluginProvenanceRecord, PluginTrustTier, PostmortemTemplate, ProductBoundary,
        ReleaseGovernancePolicy, RunProvenanceAttestation, RunbookEntry, ServiceLevelIndicators,
        ServiceLevelObjective, SupportabilityModel, SustainabilityOwnership,
    };
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        BlastRadiusSimulation, DegradedModeSimulation, IncidentAnnotationSimulation,
        IncidentModeSimulation, IncidentTimelineEvent, IncidentTimelineSimulation,
        ReadinessReviewSimulation, RepairWindowSimulation, ReplayValidationSimulation,
        ResilienceScorecardSimulation, SafeStopSimulation, WorkflowImpact,
    };

    #[test]
    fn incident_mode_accepts_reduced_action_surface_with_runbooks() {
        let simulation = IncidentModeSimulation {
            classification: IncidentClassification {
                incident_type: "scheduler-partition".to_string(),
                severity: IncidentSeverity::High,
                routing: "platform-oncall".to_string(),
            },
            dashboard: PlatformHealthDashboard {
                engine_health: 0.78,
                scheduler_health: 0.82,
                artifact_store_health: 0.75,
                auth_health: 0.88,
                policy_health: 0.84,
            },
            disabled_actions: vec!["bulk-backfill".to_string()],
            elevated_surfaces: vec!["scheduler-health".to_string(), "run-failures".to_string()],
            retained_actions: vec!["cancel".to_string(), "pause".to_string()],
            runbooks: vec![RunbookEntry {
                name: "scheduler outage".to_string(),
                trigger: "leader election instability".to_string(),
                required_evidence: vec!["timeline".to_string(), "queue snapshot".to_string()],
            }],
            supportability: SupportabilityModel {
                official_plugins: BTreeSet::from(["builtin-scheduler".to_string()]),
                supported_backends: BTreeSet::from(["local".to_string(), "remote".to_string()]),
            },
        };
        let (payload, ok) = incident_mode_payload(simulation);
        assert!(ok);
        assert_eq!(payload["incident_mode_ready"], true);
    }

    #[test]
    fn incident_mode_flags_missing_controls() {
        let simulation = IncidentModeSimulation {
            classification: IncidentClassification {
                incident_type: "artifact-store-outage".to_string(),
                severity: IncidentSeverity::Critical,
                routing: "platform-incident".to_string(),
            },
            dashboard: PlatformHealthDashboard {
                engine_health: 0.50,
                scheduler_health: 0.45,
                artifact_store_health: 0.30,
                auth_health: 0.70,
                policy_health: 0.68,
            },
            disabled_actions: Vec::new(),
            elevated_surfaces: Vec::new(),
            retained_actions: Vec::new(),
            runbooks: Vec::new(),
            supportability: SupportabilityModel {
                official_plugins: BTreeSet::new(),
                supported_backends: BTreeSet::new(),
            },
        };
        let (payload, ok) = incident_mode_payload(simulation);
        assert!(!ok);
        assert_eq!(payload["incident_mode_ready"], false);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn blast_radius_resolves_impacted_workflows_and_tenants() {
        let simulation = BlastRadiusSimulation {
            classification: IncidentClassification {
                incident_type: "policy-store-outage".to_string(),
                severity: IncidentSeverity::High,
                routing: "platform-oncall".to_string(),
            },
            failing_components: vec!["policy-store".to_string()],
            workflows: vec![
                WorkflowImpact {
                    workflow_id: "tenant-a/regulated-backfill".to_string(),
                    tenant_id: "tenant-a".to_string(),
                    artifact_ids: vec!["artifact-1".to_string(), "artifact-2".to_string()],
                    dependencies: vec!["policy-store".to_string(), "artifact-store".to_string()],
                },
                WorkflowImpact {
                    workflow_id: "tenant-b/analytics-refresh".to_string(),
                    tenant_id: "tenant-b".to_string(),
                    artifact_ids: vec!["artifact-3".to_string()],
                    dependencies: vec!["scheduler".to_string()],
                },
            ],
            service_context: BTreeMap::from([(
                "policy-store".to_string(),
                vec!["authz".to_string(), "release-gates".to_string()],
            )]),
        };
        let (payload, ok) = blast_radius_payload(simulation);
        assert!(ok);
        assert_eq!(payload["impacted_workflows"].as_array().expect("workflows").len(), 1);
        assert_eq!(payload["impacted_tenants"][0], "tenant-a");
    }

    #[test]
    fn blast_radius_flags_missing_context_or_impacts() {
        let simulation = BlastRadiusSimulation {
            classification: IncidentClassification {
                incident_type: "scheduler-drift".to_string(),
                severity: IncidentSeverity::Medium,
                routing: "platform-review".to_string(),
            },
            failing_components: vec!["scheduler".to_string(), "artifact-store".to_string()],
            workflows: vec![WorkflowImpact {
                workflow_id: "tenant-a/no-hit".to_string(),
                tenant_id: "tenant-a".to_string(),
                artifact_ids: Vec::new(),
                dependencies: vec!["authz".to_string()],
            }],
            service_context: BTreeMap::from([(
                "scheduler".to_string(),
                vec!["planner".to_string()],
            )]),
        };
        let (payload, ok) = blast_radius_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 2);
    }

    #[test]
    fn safe_stop_accepts_explicit_boundaries_and_restart_conditions() {
        let simulation = SafeStopSimulation {
            incident_id: "inc-2026-04-28-1".to_string(),
            stop_scope: vec!["scheduler".to_string(), "tenant:regulated".to_string()],
            queued_runs: 12,
            running_runs: 3,
            stop_new_schedules: true,
            stop_new_dispatch: true,
            drain_running_work: true,
            preserve_artifact_commits: true,
            restart_conditions: vec![
                "leadership stable".to_string(),
                "artifact store green".to_string(),
            ],
            runbook: RunbookEntry {
                name: "platform freeze".to_string(),
                trigger: "severe control-plane instability".to_string(),
                required_evidence: vec![
                    "scheduler fence".to_string(),
                    "queue snapshot".to_string(),
                ],
            },
            approval_required: true,
        };
        let (payload, ok) = safe_stop_payload(simulation);
        assert!(ok);
        assert_eq!(payload["safe_stop_ready"], true);
    }

    #[test]
    fn safe_stop_flags_missing_freeze_boundaries() {
        let simulation = SafeStopSimulation {
            incident_id: "inc-2026-04-28-2".to_string(),
            stop_scope: Vec::new(),
            queued_runs: 4,
            running_runs: 9,
            stop_new_schedules: false,
            stop_new_dispatch: false,
            drain_running_work: false,
            preserve_artifact_commits: false,
            restart_conditions: Vec::new(),
            runbook: RunbookEntry {
                name: "incomplete".to_string(),
                trigger: "missing".to_string(),
                required_evidence: Vec::new(),
            },
            approval_required: false,
        };
        let (payload, ok) = safe_stop_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn degraded_mode_accepts_useful_restricted_operation() {
        let simulation = DegradedModeSimulation {
            missing_dependencies: vec!["artifact-store".to_string()],
            read_only_mode: true,
            limited_submit_mode: false,
            queue_drain_mode: true,
            blocked_actions: vec!["new backfill".to_string(), "promotion".to_string()],
            available_actions: vec!["inspect".to_string(), "cancel".to_string()],
            boundary: ProductBoundary {
                platform_guarantees: vec!["existing run state remains queryable".to_string()],
                operator_responsibilities: vec![
                    "avoid promotion while artifact store is unavailable".to_string(),
                ],
            },
        };
        let (payload, ok) = degraded_mode_payload(simulation);
        assert!(ok);
        assert_eq!(payload["degraded_mode_ready"], true);
    }

    #[test]
    fn degraded_mode_flags_missing_visibility_and_boundaries() {
        let simulation = DegradedModeSimulation {
            missing_dependencies: Vec::new(),
            read_only_mode: false,
            limited_submit_mode: false,
            queue_drain_mode: false,
            blocked_actions: Vec::new(),
            available_actions: Vec::new(),
            boundary: ProductBoundary {
                platform_guarantees: Vec::new(),
                operator_responsibilities: Vec::new(),
            },
        };
        let (payload, ok) = degraded_mode_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn annotation_accepts_linked_searchable_context() {
        let simulation = IncidentAnnotationSimulation {
            incident_id: "inc-2026-04-28-3".to_string(),
            author: "platform-oncall".to_string(),
            unix_ms: 1_714_269_100_000,
            note: "artifact promotion paused until trust domain mismatch is resolved".to_string(),
            run_ids: vec!["run-1".to_string()],
            tenant_ids: vec!["tenant-a".to_string()],
            artifact_ids: vec!["artifact-7".to_string()],
            tags: vec!["promotion".to_string(), "trust-domain".to_string()],
            searchable_fields: BTreeMap::from([
                ("incident_id".to_string(), "inc-2026-04-28-3".to_string()),
                ("author".to_string(), "platform-oncall".to_string()),
                ("tenant".to_string(), "tenant-a".to_string()),
            ]),
        };
        let (payload, ok) = annotation_payload(simulation);
        assert!(ok);
        assert_eq!(payload["annotation_ready"], true);
    }

    #[test]
    fn annotation_flags_unlinked_or_unindexed_notes() {
        let simulation = IncidentAnnotationSimulation {
            incident_id: "inc-2026-04-28-4".to_string(),
            author: String::new(),
            unix_ms: 1_714_269_200_000,
            note: String::new(),
            run_ids: Vec::new(),
            tenant_ids: Vec::new(),
            artifact_ids: Vec::new(),
            tags: Vec::new(),
            searchable_fields: BTreeMap::new(),
        };
        let (payload, ok) = annotation_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn repair_window_accepts_scoped_approved_repair_plan() {
        let simulation = RepairWindowSimulation {
            incident_id: "inc-2026-04-28-5".to_string(),
            requested_by: "platform-oncall".to_string(),
            approved_by: vec!["service-owner".to_string(), "tenant-admin".to_string()],
            start_utc: "2026-04-28T11:00:00Z".to_string(),
            end_utc: "2026-04-28T12:00:00Z".to_string(),
            scope: vec!["tenant-a".to_string(), "artifact-registry".to_string()],
            run_ids: vec!["run-9".to_string()],
            tenant_ids: vec!["tenant-a".to_string()],
            repair_actions: vec![
                "rebuild output index".to_string(),
                "replay failed branch".to_string(),
            ],
            outcome_tracking: vec!["audit_event_id".to_string(), "post_check_status".to_string()],
            lifecycle_rule: LifecycleGovernanceRule {
                feature_name: "repair-window".to_string(),
                state: "approved".to_string(),
                decision_due_utc: "2026-04-28T13:00:00Z".to_string(),
            },
        };
        let (payload, ok) = repair_window_payload(simulation);
        assert!(ok);
        assert_eq!(payload["repair_window_ready"], true);
    }

    #[test]
    fn repair_window_flags_missing_scope_approval_or_tracking() {
        let simulation = RepairWindowSimulation {
            incident_id: "inc-2026-04-28-6".to_string(),
            requested_by: String::new(),
            approved_by: Vec::new(),
            start_utc: "2026-04-28T11:00:00Z".to_string(),
            end_utc: "2026-04-28T11:00:00Z".to_string(),
            scope: Vec::new(),
            run_ids: Vec::new(),
            tenant_ids: Vec::new(),
            repair_actions: Vec::new(),
            outcome_tracking: Vec::new(),
            lifecycle_rule: LifecycleGovernanceRule {
                feature_name: "repair-window".to_string(),
                state: String::new(),
                decision_due_utc: "2026-04-28T13:00:00Z".to_string(),
            },
        };
        let (payload, ok) = repair_window_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 6);
    }

    #[test]
    fn timeline_accepts_ordered_events_with_postmortem_sections() {
        let simulation = IncidentTimelineSimulation {
            incident_id: "inc-2026-04-28-7".to_string(),
            events: vec![
                IncidentTimelineEvent {
                    unix_ms: 10,
                    source: "monitoring".to_string(),
                    action: "detected scheduler latency spike".to_string(),
                    phase: "detection".to_string(),
                    run_id: None,
                    tenant_id: Some("tenant-a".to_string()),
                },
                IncidentTimelineEvent {
                    unix_ms: 20,
                    source: "operator".to_string(),
                    action: "entered safe stop".to_string(),
                    phase: "mitigation".to_string(),
                    run_id: Some("run-11".to_string()),
                    tenant_id: Some("tenant-a".to_string()),
                },
                IncidentTimelineEvent {
                    unix_ms: 30,
                    source: "operator".to_string(),
                    action: "resumed queues after repair".to_string(),
                    phase: "recovery".to_string(),
                    run_id: Some("run-11".to_string()),
                    tenant_id: Some("tenant-a".to_string()),
                },
            ],
            postmortem: PostmortemTemplate {
                required_sections: vec![
                    "summary".to_string(),
                    "impact".to_string(),
                    "timeline".to_string(),
                    "actions".to_string(),
                ],
            },
        };
        let (payload, ok) = timeline_payload(simulation);
        assert!(ok);
        assert_eq!(payload["timeline_ready"], true);
    }

    #[test]
    fn timeline_flags_missing_recovery_story() {
        let simulation = IncidentTimelineSimulation {
            incident_id: "inc-2026-04-28-8".to_string(),
            events: vec![IncidentTimelineEvent {
                unix_ms: 10,
                source: "monitoring".to_string(),
                action: "detected failure".to_string(),
                phase: "detection".to_string(),
                run_id: None,
                tenant_id: None,
            }],
            postmortem: PostmortemTemplate { required_sections: vec!["summary".to_string()] },
        };
        let (payload, ok) = timeline_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }

    fn sample_attestation(run_id: &str, trust_domain: &str) -> RunProvenanceAttestation {
        RunProvenanceAttestation {
            run_id: run_id.to_string(),
            dag_snapshot_id: "dag-snapshot-1".to_string(),
            plan_fingerprint: "plan-1".to_string(),
            policy_bundle_version: "policy-v1".to_string(),
            output_digests: vec!["sha256:abc".to_string()],
            binaries: vec![BinaryProvenanceRecord {
                component: BinaryComponent::Scheduler,
                version: "1.0.0".to_string(),
                build_id: "build-1".to_string(),
                source_revision: "rev-1".to_string(),
                build_timestamp_utc: "2026-04-28T10:00:00Z".to_string(),
            }],
            plugins: vec![PluginProvenanceRecord {
                plugin_name: "official-transfer".to_string(),
                version: "1.0.0".to_string(),
                source: "registry".to_string(),
                trust_tier: PluginTrustTier::Official,
                approved: true,
            }],
            environment: EnvironmentAttestation {
                backend: "remote".to_string(),
                capability_class: "regulated".to_string(),
                trust_domain: trust_domain.to_string(),
            },
        }
    }

    #[test]
    fn replay_validation_accepts_clean_recovery_replay() {
        let simulation = ReplayValidationSimulation {
            incident_id: "inc-2026-04-28-9".to_string(),
            baseline: sample_attestation("run-12", "prod-trust"),
            candidate: sample_attestation("run-12-replay", "prod-trust"),
            release_policy: ReleaseGovernancePolicy {
                requires_evidence_bundle: true,
                requires_compatibility_results: true,
                requires_rollback_plan: true,
            },
            outputs_match: true,
            lineage_consistent: true,
        };
        let (payload, ok) = replay_validation_payload(simulation);
        assert!(ok);
        assert_eq!(payload["replay_validation_ready"], true);
    }

    #[test]
    fn replay_validation_flags_provenance_or_policy_drift() {
        let simulation = ReplayValidationSimulation {
            incident_id: "inc-2026-04-28-10".to_string(),
            baseline: sample_attestation("run-13", "prod-trust"),
            candidate: sample_attestation("run-13-replay", "staging-trust"),
            release_policy: ReleaseGovernancePolicy {
                requires_evidence_bundle: true,
                requires_compatibility_results: false,
                requires_rollback_plan: true,
            },
            outputs_match: false,
            lineage_consistent: false,
        };
        let (payload, ok) = replay_validation_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }

    #[test]
    fn readiness_review_accepts_owned_runbook_backed_workflow() {
        let simulation = ReadinessReviewSimulation {
            workflow_id: "tenant-a/critical-refresh".to_string(),
            owners: vec!["team-data".to_string(), "team-platform".to_string()],
            runbooks: vec![RunbookEntry {
                name: "critical refresh".to_string(),
                trigger: "missed dispatch SLO".to_string(),
                required_evidence: vec!["timeline".to_string(), "retry report".to_string()],
            }],
            training: OperatorTrainingCatalog {
                interventions: vec!["pause queue".to_string(), "approve replay".to_string()],
            },
            audit_checklist: AuditReadinessChecklist {
                checks: vec!["owner confirmed".to_string(), "runbook linked".to_string()],
            },
            objective: ServiceLevelObjective {
                run_creation_latency_ms: 500.0,
                dispatch_latency_ms: 800.0,
                completion_reliability_ratio: 0.99,
                artifact_availability_ratio: 0.995,
            },
            indicators: ServiceLevelIndicators {
                measured_run_creation_latency_ms: 420.0,
                measured_dispatch_latency_ms: 600.0,
                measured_completion_reliability_ratio: 0.995,
                measured_artifact_availability_ratio: 0.998,
            },
            verification_lane: None,
            supportability: SupportabilityModel {
                official_plugins: BTreeSet::from(["official-transfer".to_string()]),
                supported_backends: BTreeSet::from(["remote".to_string(), "hpc".to_string()]),
            },
        };
        let (payload, ok) = readiness_review_payload(simulation);
        assert!(ok);
        assert_eq!(payload["readiness_review_passed"], true);
    }

    #[test]
    fn readiness_review_flags_missing_ownership_or_slo_conformance() {
        let simulation = ReadinessReviewSimulation {
            workflow_id: "tenant-b/unowned-refresh".to_string(),
            owners: Vec::new(),
            runbooks: Vec::new(),
            training: OperatorTrainingCatalog { interventions: Vec::new() },
            audit_checklist: AuditReadinessChecklist { checks: Vec::new() },
            objective: ServiceLevelObjective {
                run_creation_latency_ms: 100.0,
                dispatch_latency_ms: 100.0,
                completion_reliability_ratio: 0.99,
                artifact_availability_ratio: 0.99,
            },
            indicators: ServiceLevelIndicators {
                measured_run_creation_latency_ms: 400.0,
                measured_dispatch_latency_ms: 300.0,
                measured_completion_reliability_ratio: 0.80,
                measured_artifact_availability_ratio: 0.85,
            },
            verification_lane: None,
            supportability: SupportabilityModel {
                official_plugins: BTreeSet::new(),
                supported_backends: BTreeSet::new(),
            },
        };
        let (payload, ok) = readiness_review_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn scorecard_rewards_resilient_workflow_family() {
        let simulation = ResilienceScorecardSimulation {
            workflow_id: "tenant-a/resilient-refresh".to_string(),
            objective: ServiceLevelObjective {
                run_creation_latency_ms: 500.0,
                dispatch_latency_ms: 800.0,
                completion_reliability_ratio: 0.99,
                artifact_availability_ratio: 0.995,
            },
            indicators: ServiceLevelIndicators {
                measured_run_creation_latency_ms: 300.0,
                measured_dispatch_latency_ms: 500.0,
                measured_completion_reliability_ratio: 0.996,
                measured_artifact_availability_ratio: 0.999,
            },
            dashboard: PlatformHealthDashboard {
                engine_health: 0.92,
                scheduler_health: 0.94,
                artifact_store_health: 0.96,
                auth_health: 0.95,
                policy_health: 0.93,
            },
            gamedays: vec![GamedayScenario {
                name: "scheduler failover".to_string(),
                failure_class: "control-plane".to_string(),
                expected_outcome: "queue ownership transfers cleanly".to_string(),
            }],
            postmortem: PostmortemTemplate {
                required_sections: vec![
                    "summary".to_string(),
                    "impact".to_string(),
                    "timeline".to_string(),
                    "actions".to_string(),
                ],
            },
            invariants: None,
            verification_lane: None,
            supportability: SupportabilityModel {
                official_plugins: BTreeSet::from(["official-transfer".to_string()]),
                supported_backends: BTreeSet::from(["remote".to_string(), "hpc".to_string()]),
            },
            sustainability: SustainabilityOwnership {
                subsystem_owners: BTreeMap::from([(
                    "scheduler".to_string(),
                    "team-platform".to_string(),
                )]),
                review_routing: BTreeMap::from([(
                    "scheduler".to_string(),
                    "platform-review".to_string(),
                )]),
            },
            acceptance_board: PlatformAcceptanceBoard {
                members: vec!["platform".to_string(), "security".to_string()],
                preview_to_stable_criteria: vec![
                    "gameday green".to_string(),
                    "slo green".to_string(),
                ],
            },
            error_budget: ErrorBudgetPolicy {
                scheduler_outage_minutes_per_quarter: 20.0,
                backend_degradation_minutes_per_quarter: 60.0,
                artifact_corruption_incident_budget: 0,
            },
        };
        let (payload, ok) = scorecard_payload(simulation);
        assert!(ok);
        assert!(payload["score"].as_f64().expect("score") >= 75.0);
    }

    #[test]
    fn scorecard_penalizes_missing_resilience_foundations() {
        let simulation = ResilienceScorecardSimulation {
            workflow_id: "tenant-b/brittle-refresh".to_string(),
            objective: ServiceLevelObjective {
                run_creation_latency_ms: 100.0,
                dispatch_latency_ms: 100.0,
                completion_reliability_ratio: 0.99,
                artifact_availability_ratio: 0.99,
            },
            indicators: ServiceLevelIndicators {
                measured_run_creation_latency_ms: 500.0,
                measured_dispatch_latency_ms: 500.0,
                measured_completion_reliability_ratio: 0.70,
                measured_artifact_availability_ratio: 0.80,
            },
            dashboard: PlatformHealthDashboard {
                engine_health: 0.40,
                scheduler_health: 0.35,
                artifact_store_health: 0.30,
                auth_health: 0.60,
                policy_health: 0.55,
            },
            gamedays: Vec::new(),
            postmortem: PostmortemTemplate { required_sections: vec!["summary".to_string()] },
            invariants: Some(PlatformInvariantCatalog { invariants: Vec::new() }),
            verification_lane: Some(IntegratedVerificationLane {
                name: "thin".to_string(),
                required_domains: Vec::new(),
                required_evidence: Vec::new(),
            }),
            supportability: SupportabilityModel {
                official_plugins: BTreeSet::new(),
                supported_backends: BTreeSet::new(),
            },
            sustainability: SustainabilityOwnership {
                subsystem_owners: BTreeMap::new(),
                review_routing: BTreeMap::new(),
            },
            acceptance_board: PlatformAcceptanceBoard {
                members: Vec::new(),
                preview_to_stable_criteria: Vec::new(),
            },
            error_budget: ErrorBudgetPolicy {
                scheduler_outage_minutes_per_quarter: 0.0,
                backend_degradation_minutes_per_quarter: 0.0,
                artifact_corruption_incident_budget: 1,
            },
        };
        let (payload, ok) = scorecard_payload(simulation);
        assert!(!ok);
        assert!(payload["weaknesses"].as_array().expect("weaknesses").len() >= 5);
    }
}
