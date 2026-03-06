use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretReference {
    pub secret_id: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretSource {
    LocalDev,
    FilePath { path: String },
    Environment { key: String },
    ExternalManager { provider: String, path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretResolutionTiming {
    RuntimeOnly,
    CompileTimeAllowed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretUsageAuditEvent {
    pub secret_id: String,
    pub node_id: String,
    pub run_id: String,
    pub unix_ms: u128,
    pub access_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretMaskingPolicy {
    pub redact_logs: bool,
    pub redact_diagnostics: bool,
    pub redact_manifests: bool,
    pub redact_exports: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretScopeRule {
    pub tenant_id: Option<String>,
    pub dag_id: Option<String>,
    pub run_id: Option<String>,
    pub node_id: Option<String>,
    pub worker_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretInjectionMode {
    Env,
    FileMount,
    BackendNative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretDeliveryPolicy {
    pub allowed_modes: Vec<SecretInjectionMode>,
    pub deny_process_args: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretRotationRule {
    pub allow_latest: bool,
    pub require_pin_for_backfill: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretVersionSelection {
    pub selected_version: String,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretArtifactPolicy {
    pub allow_secret_artifacts: bool,
    pub required_classification: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretTaintRecord {
    pub node_id: String,
    pub tainted_logs: bool,
    pub tainted_diagnostics: bool,
    pub tainted_artifacts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensitiveArtifactClass {
    SecretDerived,
    Regulated,
    InternalRestricted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensitiveArtifactRestriction {
    pub class: SensitiveArtifactClass,
    pub min_retention_days: u32,
    pub export_requires_approval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecureWorkspaceRule {
    pub secure_temp_cleanup: bool,
    pub remove_secret_mounts_on_exit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecureTeardownPolicy {
    pub wipe_env_on_cancel: bool,
    pub wipe_files_on_cancel: bool,
    pub teardown_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecureExecutionMode {
    pub enabled: bool,
    pub environment: String,
    pub strict_policy_bundle: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretIntegrationReadiness {
    pub source_connected: bool,
    pub masking_enabled: bool,
    pub audit_enabled: bool,
    pub strict_mode_supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretLeakIncident {
    pub incident_id: String,
    pub detected_in: String,
    pub run_id: Option<String>,
    pub containment_actions: Vec<String>,
}

pub fn secret_scope_allows(scope: &SecretScopeRule, requested: &SecretScopeRule) -> bool {
    let field_match = |a: &Option<String>, b: &Option<String>| a.is_none() || a == b;
    field_match(&scope.tenant_id, &requested.tenant_id)
        && field_match(&scope.dag_id, &requested.dag_id)
        && field_match(&scope.run_id, &requested.run_id)
        && field_match(&scope.node_id, &requested.node_id)
        && field_match(&scope.worker_id, &requested.worker_id)
}

pub fn validate_secret_delivery_mode(
    mode: &SecretInjectionMode,
    policy: &SecretDeliveryPolicy,
) -> bool {
    policy.allowed_modes.iter().any(|m| m == mode)
}

pub fn select_secret_version(
    available_versions: &[String],
    pinned_version: Option<&str>,
    rotation: &SecretRotationRule,
    is_backfill: bool,
) -> Option<SecretVersionSelection> {
    if let Some(pin) = pinned_version {
        if available_versions.iter().any(|v| v == pin) {
            return Some(SecretVersionSelection {
                selected_version: pin.to_string(),
                pinned: true,
            });
        }
        return None;
    }
    if is_backfill && rotation.require_pin_for_backfill {
        return None;
    }
    if rotation.allow_latest {
        let latest = available_versions.iter().max()?.clone();
        return Some(SecretVersionSelection {
            selected_version: latest,
            pinned: false,
        });
    }
    None
}

pub fn should_materialize_secret_artifact(policy: &SecretArtifactPolicy, classification: Option<&str>) -> bool {
    policy.allow_secret_artifacts
        && policy
            .required_classification
            .as_ref()
            .map(|required| Some(required.as_str()) == classification)
            .unwrap_or(true)
}

pub fn redact_secret_payload(payload: &str, secrets: &[String]) -> String {
    let mut redacted = payload.to_string();
    for secret in secrets {
        redacted = redacted.replace(secret, "***REDACTED***");
    }
    redacted
}

pub fn taint_from_secret_usage(
    used_secret: bool,
    produced_artifacts: bool,
    produced_logs: bool,
) -> SecretTaintRecord {
    SecretTaintRecord {
        node_id: "unknown".to_string(),
        tainted_logs: used_secret && produced_logs,
        tainted_diagnostics: used_secret,
        tainted_artifacts: used_secret && produced_artifacts,
    }
}

pub fn secure_mode_effective(environment: &str, mode: &SecureExecutionMode) -> bool {
    mode.enabled && mode.environment == environment
}

pub fn leak_conformance_check(outputs: &[String]) -> bool {
    !outputs.iter().any(|line| {
        let l = line.to_ascii_lowercase();
        l.contains("secret=") || l.contains("token=") || l.contains("password=")
    })
}

pub fn secret_readiness(
    sources: &[SecretSource],
    masking_policy: &SecretMaskingPolicy,
    audit_events: &[SecretUsageAuditEvent],
    secure_mode_supported: bool,
) -> SecretIntegrationReadiness {
    SecretIntegrationReadiness {
        source_connected: !sources.is_empty(),
        masking_enabled: masking_policy.redact_logs
            && masking_policy.redact_diagnostics
            && masking_policy.redact_manifests
            && masking_policy.redact_exports,
        audit_enabled: !audit_events.is_empty(),
        strict_mode_supported: secure_mode_supported,
    }
}

pub fn incident_response_actions(incident: &SecretLeakIncident) -> BTreeSet<String> {
    incident
        .containment_actions
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
}

pub fn secure_cleanup_required(rule: &SecureWorkspaceRule, teardown: &SecureTeardownPolicy) -> bool {
    rule.secure_temp_cleanup
        && rule.remove_secret_mounts_on_exit
        && teardown.wipe_env_on_cancel
        && teardown.wipe_files_on_cancel
}

pub fn summarize_sensitive_classes(
    restrictions: &[SensitiveArtifactRestriction],
) -> BTreeMap<String, (u32, bool)> {
    let mut map = BTreeMap::new();
    for r in restrictions {
        map.insert(
            format!("{:?}", r.class),
            (r.min_retention_days, r.export_requires_approval),
        );
    }
    map
}
