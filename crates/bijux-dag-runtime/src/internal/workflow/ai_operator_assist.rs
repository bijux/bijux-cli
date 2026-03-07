use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvestigationBundle {
    pub bundle_id: String,
    pub run_id: String,
    pub evidence_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailureSummary {
    pub run_id: String,
    pub failed_nodes: Vec<String>,
    pub stuck_nodes: Vec<String>,
    pub policy_denials: Vec<String>,
    pub artifact_mismatches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticsAnswer {
    pub question: String,
    pub answer: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafeOperatorAction {
    Replay,
    Verify,
    InspectLineage,
    SuppressSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuggestedAction {
    pub action: SafeOperatorAction,
    pub reason: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WhatChangedSummary {
    pub baseline_run_id: String,
    pub current_run_id: String,
    pub differences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerReviewSummary {
    pub plan_id: String,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduleAnomalySummary {
    pub trigger_volume_delta: f64,
    pub latency_delta: f64,
    pub queue_pressure_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactAnomalySummary {
    pub size_delta_ratio: f64,
    pub schema_mismatch: bool,
    pub suspected_lineage_gap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RootCauseDomainHint {
    pub likely_domain: String,
    pub confidence: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafeActionGuardrail {
    pub policy_compatible: bool,
    pub permission_compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayRecommendation {
    pub target_run_id: String,
    pub minimal_recompute_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentSimilarityResult {
    pub incident_id: String,
    pub similarity_score: u8,
    pub matching_signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostmortemSeed {
    pub run_id: String,
    pub sections: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservabilityAnomalySignal {
    pub retries_spike: f64,
    pub hang_rate: f64,
    pub cache_error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceCitation {
    pub source: String,
    pub line_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorReviewDecision {
    pub suggestion_id: String,
    pub accepted: bool,
    pub annotation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivacyRedactionPolicy {
    pub redact_secrets: bool,
    pub redact_pii: bool,
    pub redact_tenant_sensitive_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecommendationSimulationResult {
    pub scenario_id: String,
    pub recommendation_correct: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiAssistMaturityLevel {
    DiagnosticsOnly,
    EvidenceGuidedSuggestions,
    GuardedRecommendations,
}

pub fn build_investigation_bundle(run_id: &str) -> InvestigationBundle {
    InvestigationBundle {
        bundle_id: format!("bundle-{run_id}"),
        run_id: run_id.to_string(),
        evidence_sections: vec![
            "run-events".to_string(),
            "lineage-snapshot".to_string(),
            "policy-evaluations".to_string(),
            "artifact-diagnostics".to_string(),
        ],
    }
}

pub fn answer_failure_question(summary: &FailureSummary) -> DiagnosticsAnswer {
    let mut evidence = Vec::new();
    if !summary.failed_nodes.is_empty() {
        evidence.push(format!("failed-nodes={}", summary.failed_nodes.join(",")));
    }
    if !summary.policy_denials.is_empty() {
        evidence.push(format!(
            "policy-denials={}",
            summary.policy_denials.join(",")
        ));
    }
    DiagnosticsAnswer {
        question: "why did this fail".to_string(),
        answer: "failure involved node execution and/or policy constraints".to_string(),
        evidence,
    }
}

pub fn guardrail_allows(guardrail: &SafeActionGuardrail) -> bool {
    guardrail.policy_compatible && guardrail.permission_compatible
}

pub fn recommend_safe_actions(
    summary: &FailureSummary,
    guardrail: &SafeActionGuardrail,
) -> Vec<SuggestedAction> {
    if !guardrail_allows(guardrail) {
        return Vec::new();
    }

    let mut out = Vec::new();
    if !summary.failed_nodes.is_empty() {
        out.push(SuggestedAction {
            action: SafeOperatorAction::Replay,
            reason: "failed nodes detected".to_string(),
            evidence: summary.failed_nodes.clone(),
        });
        out.push(SuggestedAction {
            action: SafeOperatorAction::Verify,
            reason: "verify output contracts before rerun".to_string(),
            evidence: vec!["contract-surface".to_string()],
        });
    }
    if !summary.artifact_mismatches.is_empty() {
        out.push(SuggestedAction {
            action: SafeOperatorAction::InspectLineage,
            reason: "artifact mismatch detected".to_string(),
            evidence: summary.artifact_mismatches.clone(),
        });
    }
    out
}

pub fn anomaly_detected(signal: &ObservabilityAnomalySignal, threshold: f64) -> bool {
    signal.retries_spike >= threshold
        || signal.hang_rate >= threshold
        || signal.cache_error_rate >= threshold
}

pub fn build_postmortem_seed(summary: &FailureSummary) -> PostmortemSeed {
    let mut sections = BTreeMap::new();
    sections.insert(
        "impact".to_string(),
        format!("failed_nodes={}", summary.failed_nodes.len()),
    );
    sections.insert(
        "timeline".to_string(),
        "see investigation bundle timeline".to_string(),
    );
    sections.insert(
        "root-cause-hypothesis".to_string(),
        "pending operator review".to_string(),
    );
    PostmortemSeed {
        run_id: summary.run_id.clone(),
        sections,
    }
}

pub fn redact_for_ai_export(
    bundle: &InvestigationBundle,
    policy: &PrivacyRedactionPolicy,
) -> InvestigationBundle {
    let mut sections = bundle.evidence_sections.clone();
    if policy.redact_secrets {
        sections.push("redacted-secrets".to_string());
    }
    if policy.redact_pii {
        sections.push("redacted-pii".to_string());
    }
    if policy.redact_tenant_sensitive_metadata {
        sections.push("redacted-tenant-metadata".to_string());
    }

    InvestigationBundle {
        bundle_id: bundle.bundle_id.clone(),
        run_id: bundle.run_id.clone(),
        evidence_sections: sections,
    }
}

pub fn suggestion_quality(simulations: &[RecommendationSimulationResult]) -> f64 {
    if simulations.is_empty() {
        return 0.0;
    }
    let passed = simulations
        .iter()
        .filter(|item| item.recommendation_correct)
        .count();
    passed as f64 / simulations.len() as f64
}

pub fn next_maturity_level(
    current: AiAssistMaturityLevel,
    quality_score: f64,
    guardrails_strict: bool,
) -> AiAssistMaturityLevel {
    match current {
        AiAssistMaturityLevel::DiagnosticsOnly if quality_score >= 0.7 => {
            AiAssistMaturityLevel::EvidenceGuidedSuggestions
        }
        AiAssistMaturityLevel::EvidenceGuidedSuggestions
            if quality_score >= 0.85 && guardrails_strict =>
        {
            AiAssistMaturityLevel::GuardedRecommendations
        }
        _ => current,
    }
}

pub fn root_cause_domain_hints(summary: &FailureSummary) -> Vec<RootCauseDomainHint> {
    let mut hints = Vec::new();
    if !summary.failed_nodes.is_empty() {
        hints.push(RootCauseDomainHint {
            likely_domain: "backend-issue".to_string(),
            confidence: 70,
        });
    }
    if !summary.policy_denials.is_empty() {
        hints.push(RootCauseDomainHint {
            likely_domain: "policy-issue".to_string(),
            confidence: 80,
        });
    }
    if !summary.artifact_mismatches.is_empty() {
        hints.push(RootCauseDomainHint {
            likely_domain: "artifact-issue".to_string(),
            confidence: 75,
        });
    }
    hints
}
