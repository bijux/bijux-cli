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

use bijux_dag_runtime::{
    incident_response_actions, leak_conformance_check, redact_secret_payload, secret_scope_allows,
    secure_mode_effective, select_secret_version, validate_secret_delivery_mode,
    SecretDeliveryPolicy, SecretInjectionMode, SecretLeakIncident, SecretRotationRule,
    SecretScopeRule, SecureExecutionMode,
};

#[test]
fn secret_scope_and_delivery_mode_contracts_are_enforced() {
    let policy_scope = SecretScopeRule {
        tenant_id: Some("tenant_alpha".to_string()),
        dag_id: Some("dag-a".to_string()),
        run_id: None,
        node_id: None,
        worker_id: None,
    };
    let request_scope = SecretScopeRule {
        tenant_id: Some("tenant_alpha".to_string()),
        dag_id: Some("dag-a".to_string()),
        run_id: Some("run-1".to_string()),
        node_id: None,
        worker_id: None,
    };
    assert!(secret_scope_allows(&policy_scope, &request_scope));

    let delivery = SecretDeliveryPolicy {
        allowed_modes: vec![SecretInjectionMode::Env, SecretInjectionMode::FileMount],
        deny_process_args: true,
    };
    assert!(validate_secret_delivery_mode(&SecretInjectionMode::Env, &delivery));
    assert!(!validate_secret_delivery_mode(&SecretInjectionMode::BackendNative, &delivery));
}

#[test]
fn secret_versioning_redaction_and_leak_checks_are_stable() {
    let selected = select_secret_version(
        &["v1".to_string(), "v2".to_string()],
        None,
        &SecretRotationRule { allow_latest: true, require_pin_for_backfill: false },
        false,
    )
    .expect("latest selected");
    assert_eq!(selected.selected_version, "v2");

    let redacted = redact_secret_payload("token=abc123", &["abc123".to_string()]);
    assert_eq!(redacted, "token=***REDACTED***");
    assert!(leak_conformance_check(&["status=ok".to_string()]));
    assert!(!leak_conformance_check(&["password=cleartext".to_string()]));
}

#[test]
fn secure_mode_and_incident_actions_are_explicit() {
    assert!(secure_mode_effective(
        "prod",
        &SecureExecutionMode {
            enabled: true,
            environment: "prod".to_string(),
            strict_policy_bundle: "strict-prod".to_string(),
        }
    ));
    let incident = SecretLeakIncident {
        incident_id: "sec-1".to_string(),
        detected_in: "stderr".to_string(),
        run_id: Some("run-1".to_string()),
        containment_actions: vec!["revoke-credentials".to_string(), "quarantine-run".to_string()],
    };
    let actions = incident_response_actions(&incident);
    assert!(actions.contains("revoke-credentials"));
    assert!(actions.contains("quarantine-run"));
}
