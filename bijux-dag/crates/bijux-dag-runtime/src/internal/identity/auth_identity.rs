use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthProvider {
    LocalDev,
    ServiceToken,
    OidcFuture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticationBoundary {
    pub provider: AuthProvider,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityPrincipalKind {
    User,
    Service,
    Scheduler,
    Worker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityPrincipal {
    pub principal_id: String,
    pub kind: IdentityPrincipalKind,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialScope {
    pub cli: bool,
    pub api_client: bool,
    pub scheduler: bool,
    pub worker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialLifecycle {
    pub issued_unix_ms: u128,
    pub expires_unix_ms: u128,
    pub renewable: bool,
    pub max_renewals: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerCredentialBinding {
    pub worker_id: String,
    pub lease_id: String,
    pub run_scope: Option<String>,
    pub expires_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialProvenanceRecord {
    pub action: String,
    pub principal_id: String,
    pub credential_class: String,
    pub unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRevocation {
    pub principal_id: String,
    pub reason: String,
    pub revoked_unix_ms: u128,
    pub propagate_to_running_operations: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerBootstrapTrustFlow {
    pub worker_id: String,
    pub enrollment_token_id: String,
    pub trust_domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerBootstrapTrustFlow {
    pub scheduler_id: String,
    pub replica_group: String,
    pub trust_domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginTrustRegistration {
    pub plugin_name: String,
    pub plugin_version: String,
    pub approved_by: String,
    pub approval_ticket: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSigningIdentity {
    pub signer_principal_id: String,
    pub trust_domain: String,
    pub algorithm: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustDomain {
    pub tenant: String,
    pub environment: String,
    pub execution_backend: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutualAuthDesignNote {
    pub control_plane_identity: String,
    pub worker_identity: String,
    pub transport: String,
    pub requirement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalDevAuthBypassRule {
    pub enabled: bool,
    pub environment: String,
    pub marker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuthenticationEventKind {
    Login,
    Refresh,
    Revoke,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticationEvent {
    pub kind: AuthenticationEventKind,
    pub principal_id: String,
    pub unix_ms: u128,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialStorageGuideline {
    pub target: String,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityProviderCompatibilityRule {
    pub from_provider: String,
    pub to_provider: String,
    pub preserves_subject_id: bool,
    pub preserves_audit_chain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustHealthReport {
    pub active_identities: usize,
    pub credential_classes: Vec<String>,
    pub policy_baselines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityFederationReadiness {
    pub local_auth_isolated: bool,
    pub revocation_propagation_supported: bool,
    pub short_lived_worker_creds_supported: bool,
    pub audit_events_complete: bool,
}

pub fn credential_is_expired(now_unix_ms: u128, lifecycle: &CredentialLifecycle) -> bool {
    now_unix_ms >= lifecycle.expires_unix_ms
}

pub fn can_renew_credential(renewal_count: u32, lifecycle: &CredentialLifecycle) -> bool {
    lifecycle.renewable && renewal_count < lifecycle.max_renewals
}

pub fn revoked_principals_set(revocations: &[CredentialRevocation]) -> BTreeSet<String> {
    revocations
        .iter()
        .map(|r| r.principal_id.clone())
        .collect::<BTreeSet<_>>()
}

pub fn local_dev_bypass_allowed(environment: &str, rule: &LocalDevAuthBypassRule) -> bool {
    rule.enabled && environment == rule.environment
}

pub fn trust_health_report(
    principals: &[IdentityPrincipal],
    credential_classes: &[String],
    policy_baselines: &[String],
) -> TrustHealthReport {
    let mut classes = credential_classes.to_vec();
    classes.sort();
    classes.dedup();
    let mut baselines = policy_baselines.to_vec();
    baselines.sort();
    baselines.dedup();
    TrustHealthReport {
        active_identities: principals.len(),
        credential_classes: classes,
        policy_baselines: baselines,
    }
}

pub fn migrate_identity_provider_compatible(rule: &IdentityProviderCompatibilityRule) -> bool {
    rule.preserves_subject_id && rule.preserves_audit_chain
}

pub fn readiness_for_federation(
    local_bypass_rules: &[LocalDevAuthBypassRule],
    revocation_supported: bool,
    short_lived_worker_creds_supported: bool,
    auth_events: &[AuthenticationEvent],
) -> IdentityFederationReadiness {
    let local_auth_isolated = local_bypass_rules
        .iter()
        .all(|r| r.environment == "local" || !r.enabled);
    let kinds: BTreeSet<_> = auth_events.iter().map(|e| e.kind.clone()).collect();
    let audit_events_complete = kinds.contains(&AuthenticationEventKind::Login)
        && kinds.contains(&AuthenticationEventKind::Refresh)
        && kinds.contains(&AuthenticationEventKind::Revoke)
        && kinds.contains(&AuthenticationEventKind::Failure);
    IdentityFederationReadiness {
        local_auth_isolated,
        revocation_propagation_supported: revocation_supported,
        short_lived_worker_creds_supported,
        audit_events_complete,
    }
}

pub fn credential_scopes_matrix(scope: &CredentialScope) -> BTreeMap<String, bool> {
    BTreeMap::from([
        ("cli".to_string(), scope.cli),
        ("api_client".to_string(), scope.api_client),
        ("scheduler".to_string(), scope.scheduler),
        ("worker".to_string(), scope.worker),
    ])
}
