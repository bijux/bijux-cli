use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::simulated_platform::{
    anomaly_detected, answer_failure_question, build_investigation_bundle, build_postmortem_seed,
    guardrail_allows, next_maturity_level, recommend_safe_actions, redact_for_ai_export,
    root_cause_domain_hints, suggestion_quality, AiAssistMaturityLevel, FailureSummary,
    ObservabilityAnomalySignal, PrivacyRedactionPolicy, RecommendationSimulationResult,
    SafeActionGuardrail,
};

fn load_failure_summary() -> FailureSummary {
    let raw = std::fs::read_to_string("tests/fixtures/ai_assist/failure_summary.json")
        .expect("failure summary fixture");
    serde_json::from_str(&raw).expect("valid failure summary fixture")
}

#[test]
fn diagnostics_and_suggestions_are_evidence_backed() {
    let summary = load_failure_summary();
    let answer = answer_failure_question(&summary);
    assert_eq!(answer.question, "why did this fail");
    assert!(!answer.evidence.is_empty());

    let guardrail = SafeActionGuardrail { policy_compatible: true, permission_compatible: true };
    let suggestions = recommend_safe_actions(&summary, &guardrail);
    assert!(!suggestions.is_empty());
}

#[test]
fn guardrails_block_suggestions_when_permissions_or_policy_fail() {
    let summary = load_failure_summary();
    let blocked = SafeActionGuardrail { policy_compatible: false, permission_compatible: true };
    assert!(!guardrail_allows(&blocked));
    assert!(recommend_safe_actions(&summary, &blocked).is_empty());
}

#[test]
fn anomaly_detection_and_postmortem_seed_generation_work() {
    let signal =
        ObservabilityAnomalySignal { retries_spike: 0.8, hang_rate: 0.2, cache_error_rate: 0.1 };
    assert!(anomaly_detected(&signal, 0.7));

    let summary = load_failure_summary();
    let seed = build_postmortem_seed(&summary);
    assert_eq!(seed.run_id, "run-2026-03-08-001");
    assert!(seed.sections.contains_key("impact"));
}

#[test]
fn redaction_and_quality_simulation_support_operator_review() {
    let bundle = build_investigation_bundle("run-2026-03-08-001");
    let policy = PrivacyRedactionPolicy {
        redact_secrets: true,
        redact_pii: true,
        redact_tenant_sensitive_metadata: true,
    };
    let redacted = redact_for_ai_export(&bundle, &policy);
    assert!(redacted.evidence_sections.iter().any(|item| item == "redacted-secrets"));

    let simulations = vec![
        RecommendationSimulationResult {
            scenario_id: "inc-1".to_string(),
            recommendation_correct: true,
            note: "good".to_string(),
        },
        RecommendationSimulationResult {
            scenario_id: "inc-2".to_string(),
            recommendation_correct: false,
            note: "bad".to_string(),
        },
    ];
    let quality = suggestion_quality(&simulations);
    assert_eq!(quality, 0.5);
}

#[test]
fn maturity_progression_and_root_cause_hints_are_stable() {
    let next = next_maturity_level(AiAssistMaturityLevel::DiagnosticsOnly, 0.8, true);
    assert_eq!(next, AiAssistMaturityLevel::EvidenceGuidedSuggestions);

    let final_level = next_maturity_level(next, 0.9, true);
    assert_eq!(final_level, AiAssistMaturityLevel::GuardedRecommendations);

    let hints = root_cause_domain_hints(&load_failure_summary());
    assert!(hints.iter().any(|hint| hint.likely_domain == "policy-issue"));
    assert!(hints.iter().any(|hint| hint.likely_domain == "artifact-issue"));
}
