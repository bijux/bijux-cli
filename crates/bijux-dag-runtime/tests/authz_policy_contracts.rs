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
    builtin_role_definitions, decision_cache_key, evaluate_authorization_acceptance,
    evaluate_dry_run, invalidate_decision_cache, is_action_allowed_in_environment,
    validate_custom_role, Action, ActionKind, CustomRoleDefinition, DecisionType,
    EnvironmentAuthorizationRule, PolicyDecisionCache, PolicyDecisionCacheEntry,
    PolicyEvaluationRequest, ResourceKind, ResourceRef, ResourceScope, SubjectIdentity,
    SubjectKind,
};

#[test]
fn role_and_custom_role_contracts_are_validated() {
    let roles = builtin_role_definitions();
    assert!(!roles.is_empty());

    let invalid = CustomRoleDefinition {
        role_name: "".to_string(),
        permissions: vec!["run.submit".to_string()],
    };
    assert!(validate_custom_role(&invalid).is_err());

    let valid = CustomRoleDefinition {
        role_name: "release_operator".to_string(),
        permissions: vec!["run.submit".to_string(), "run.cancel".to_string()],
    };
    assert!(validate_custom_role(&valid).is_ok());
}

#[test]
fn environment_and_dry_run_decisions_are_deterministic() {
    let request = PolicyEvaluationRequest {
        request_id: "req-1".to_string(),
        subject: SubjectIdentity {
            subject_id: "user-a".to_string(),
            kind: SubjectKind::User,
            tenant_id: Some("tenant_alpha".to_string()),
        },
        action: Action { name: "run.submit".to_string(), kind: ActionKind::Execute },
        resource: ResourceRef {
            kind: ResourceKind::Run,
            id: "run_1".to_string(),
            tenant_id: Some("tenant_alpha".to_string()),
        },
        scope: ResourceScope::Tenant { tenant_id: "tenant_alpha".to_string() },
        environment: "prod".to_string(),
    };

    let rules = vec![EnvironmentAuthorizationRule {
        environment: "prod".to_string(),
        denied_actions: vec!["policy.manage".to_string()],
    }];
    assert!(is_action_allowed_in_environment("run.submit", "prod", &rules));
    assert!(!is_action_allowed_in_environment("policy.manage", "prod", &rules));

    let dry_run = evaluate_dry_run(&request, &["run.submit".to_string()], "2026.03");
    assert!(dry_run.would_allow);
}

#[test]
fn cache_invalidation_and_acceptance_checks_hold() {
    let request = PolicyEvaluationRequest {
        request_id: "req-2".to_string(),
        subject: SubjectIdentity {
            subject_id: "worker-1".to_string(),
            kind: SubjectKind::Worker,
            tenant_id: Some("tenant_alpha".to_string()),
        },
        action: Action { name: "artifact.upload".to_string(), kind: ActionKind::Write },
        resource: ResourceRef {
            kind: ResourceKind::Artifact,
            id: "a1".to_string(),
            tenant_id: Some("tenant_alpha".to_string()),
        },
        scope: ResourceScope::Run {
            tenant_id: "tenant_alpha".to_string(),
            run_id: "run_1".to_string(),
        },
        environment: "staging".to_string(),
    };
    let key = decision_cache_key(&request, "2026.03");
    let mut cache = PolicyDecisionCache {
        entries: vec![
            PolicyDecisionCacheEntry {
                cache_key: key.clone(),
                decision: DecisionType::Allow,
                policy_bundle_version: "2026.03".to_string(),
            },
            PolicyDecisionCacheEntry {
                cache_key: "old".to_string(),
                decision: DecisionType::Deny,
                policy_bundle_version: "2026.02".to_string(),
            },
        ],
    };
    invalidate_decision_cache(&mut cache, "2026.03");
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.entries[0].cache_key, key);

    let acceptance = evaluate_authorization_acceptance(
        &[
            ("run.submit".to_string(), DecisionType::Allow),
            ("platform.administer".to_string(), DecisionType::Deny),
        ],
        &[true, true],
    );
    assert!(acceptance.no_cross_tenant_escalation);
}
