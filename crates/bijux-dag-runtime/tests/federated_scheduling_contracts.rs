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
    cross_domain_replay_safe, default_federation_maturity_matrix, delegation_allowed,
    domain_healthy, federation_conformance_passes, select_delegation_failure_action,
    trust_tier_allows_domain, CrossDomainReplaySafety, DelegationFailureAction,
    DelegationFailurePolicy, DomainHealthSnapshot, FederatedConformanceGate,
    InterSchedulerFlowControl, SchedulerDomainId, TrustTierRoutingRule,
};
use std::collections::BTreeSet;

fn load_health() -> Vec<DomainHealthSnapshot> {
    let raw = std::fs::read_to_string("tests/fixtures/federation/domain_health.json")
        .expect("domain health fixture");
    serde_json::from_str(&raw).expect("valid domain health fixture")
}

#[test]
fn health_propagation_blocks_unhealthy_domain() {
    let health = load_health();
    assert!(domain_healthy(&SchedulerDomainId("eu-core".to_string()), &health));
    assert!(!domain_healthy(&SchedulerDomainId("us-burst".to_string()), &health));
}

#[test]
fn flow_control_limits_delegation_storms() {
    let flow = InterSchedulerFlowControl {
        source_domain: SchedulerDomainId("eu-core".to_string()),
        target_domain: SchedulerDomainId("us-burst".to_string()),
        max_inflight_delegations: 10,
        max_delegations_per_minute: 120,
    };

    assert!(delegation_allowed(&flow, 4, 60));
    assert!(!delegation_allowed(&flow, 11, 60));
    assert!(!delegation_allowed(&flow, 4, 121));
}

#[test]
fn replay_safety_requires_all_compatibility_domains() {
    let safe = CrossDomainReplaySafety {
        artifact_compatible: true,
        policy_compatible: true,
        backend_compatible: true,
    };
    let unsafe_case = CrossDomainReplaySafety { backend_compatible: false, ..safe };

    assert!(cross_domain_replay_safe(&safe));
    assert!(!cross_domain_replay_safe(&unsafe_case));
}

#[test]
fn delegation_failure_policy_selects_action_by_failure_class() {
    let policy = DelegationFailurePolicy {
        transient_action: DelegationFailureAction::RetrySameDomain,
        persistent_action: DelegationFailureAction::Quarantine,
    };

    assert_eq!(
        select_delegation_failure_action(&policy, false),
        DelegationFailureAction::RetrySameDomain
    );
    assert_eq!(
        select_delegation_failure_action(&policy, true),
        DelegationFailureAction::Quarantine
    );
}

#[test]
fn trust_tier_routing_restricts_sensitive_workloads() {
    let rule = TrustTierRoutingRule {
        min_trust_tier: "high".to_string(),
        allowed_domains: BTreeSet::from([SchedulerDomainId("eu-core".to_string())]),
    };

    assert!(trust_tier_allows_domain(&rule, &SchedulerDomainId("eu-core".to_string())));
    assert!(!trust_tier_allows_domain(&rule, &SchedulerDomainId("us-burst".to_string())));
}

#[test]
fn federation_conformance_requires_lineage_routing_and_audit() {
    let gate = FederatedConformanceGate {
        lineage_auditable: true,
        routing_deterministic: true,
        audit_events_complete: true,
    };
    assert!(federation_conformance_passes(&gate));
}

#[test]
fn maturity_matrix_covers_domain_progression() {
    let matrix = default_federation_maturity_matrix();
    assert!(matrix.overflow_peering.contains("deterministic delegation"));
}
