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
    can_renew_credential, credential_is_expired, local_dev_bypass_allowed,
    readiness_for_federation, trust_health_report, AuthProvider, AuthenticationBoundary,
    AuthenticationEvent, AuthenticationEventKind, CredentialLifecycle, IdentityPrincipal,
    IdentityPrincipalKind, IdentityProviderCompatibilityRule, LocalDevAuthBypassRule,
};

#[test]
fn credential_lifecycle_contracts_enforce_expiry_and_renewal() {
    let lifecycle = CredentialLifecycle {
        issued_unix_ms: 10,
        expires_unix_ms: 20,
        renewable: true,
        max_renewals: 2,
    };
    assert!(!credential_is_expired(19, &lifecycle));
    assert!(credential_is_expired(20, &lifecycle));
    assert!(can_renew_credential(1, &lifecycle));
    assert!(!can_renew_credential(2, &lifecycle));
}

#[test]
fn trust_health_and_local_bypass_contracts_are_explicit() {
    let principals = vec![
        IdentityPrincipal {
            principal_id: "user-a".to_string(),
            kind: IdentityPrincipalKind::User,
            tenant_id: Some("tenant_alpha".to_string()),
        },
        IdentityPrincipal {
            principal_id: "worker-1".to_string(),
            kind: IdentityPrincipalKind::Worker,
            tenant_id: Some("tenant_alpha".to_string()),
        },
    ];
    let report = trust_health_report(
        &principals,
        &["local-token".to_string(), "service-token".to_string()],
        &["baseline-2026-03".to_string()],
    );
    assert_eq!(report.active_identities, 2);
    assert!(local_dev_bypass_allowed(
        "local",
        &LocalDevAuthBypassRule {
            enabled: true,
            environment: "local".to_string(),
            marker: "dev-only".to_string(),
        }
    ));
}

#[test]
fn federation_readiness_requires_full_auth_event_audit() {
    let events = vec![
        AuthenticationEvent {
            kind: AuthenticationEventKind::Login,
            principal_id: "user-a".to_string(),
            unix_ms: 1,
            reason: None,
        },
        AuthenticationEvent {
            kind: AuthenticationEventKind::Refresh,
            principal_id: "user-a".to_string(),
            unix_ms: 2,
            reason: None,
        },
        AuthenticationEvent {
            kind: AuthenticationEventKind::Revoke,
            principal_id: "user-a".to_string(),
            unix_ms: 3,
            reason: Some("manual".to_string()),
        },
        AuthenticationEvent {
            kind: AuthenticationEventKind::Failure,
            principal_id: "user-bad".to_string(),
            unix_ms: 4,
            reason: Some("expired".to_string()),
        },
    ];
    let readiness = readiness_for_federation(
        &[LocalDevAuthBypassRule {
            enabled: true,
            environment: "local".to_string(),
            marker: "dev-only".to_string(),
        }],
        true,
        true,
        &events,
    );
    assert!(readiness.audit_events_complete);
    assert!(readiness.local_auth_isolated);

    let compatibility = IdentityProviderCompatibilityRule {
        from_provider: "local".to_string(),
        to_provider: "oidc".to_string(),
        preserves_subject_id: true,
        preserves_audit_chain: true,
    };
    assert!(bijux_dag_runtime::simulated_platform::migrate_identity_provider_compatible(
        &compatibility
    ));
}

#[test]
fn authentication_boundary_is_typed() {
    let boundary = AuthenticationBoundary {
        provider: AuthProvider::ServiceToken,
        issuer: "bijux-control-plane".to_string(),
        audience: "bijux-workers".to_string(),
    };
    assert_eq!(boundary.issuer, "bijux-control-plane");
}
