use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

impl TenantId {
    pub fn parse(value: &str) -> Result<Self, String> {
        if value.trim().is_empty() {
            return Err("tenant id must not be empty".to_string());
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("tenant id contains invalid characters".to_string());
        }
        Ok(Self(value.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantScopedDagName {
    pub tenant_id: TenantId,
    pub namespace: String,
    pub logical_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantOwnershipMetadata {
    pub tenant_id: TenantId,
    pub owner: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantConfigOverlay {
    pub tenant_id: TenantId,
    pub values: BTreeMap<String, String>,
    pub overrides: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantQueueIsolationPolicy {
    pub tenant_id: TenantId,
    pub queue_names: Vec<String>,
    pub hard_isolation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantConcurrencyQuota {
    pub tenant_id: TenantId,
    pub max_runs: u32,
    pub max_nodes: u32,
    pub max_backfills: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantResourceBudget {
    pub tenant_id: TenantId,
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub storage_mb: u64,
    pub artifact_volume_mb: u64,
    pub schedule_pressure_limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantRetentionPolicy {
    pub tenant_id: TenantId,
    pub artifact_ttl_days: u32,
    pub logs_ttl_days: u32,
    pub audit_ttl_days: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantPolicyBundleRef {
    pub tenant_id: TenantId,
    pub policy_bundle_id: String,
    pub policy_bundle_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantObservabilityView {
    pub tenant_id: TenantId,
    pub visible_run_ids: Vec<String>,
    pub visible_artifact_ids: Vec<String>,
    pub visible_metric_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantLineageScope {
    pub tenant_id: TenantId,
    pub allowed_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantRegistryPartition {
    pub tenant_id: TenantId,
    pub storage_partition: String,
    pub index_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantSecretScope {
    pub tenant_id: TenantId,
    pub allowed_secret_prefixes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantPluginAllowlist {
    pub tenant_id: TenantId,
    pub allowed_plugins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantEnvironmentOverlay {
    pub tenant_id: TenantId,
    pub backend_classes: Vec<String>,
    pub executor_classes: Vec<String>,
    pub storage_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantSchedulerAdmission {
    pub tenant_id: TenantId,
    pub max_enqueued_runs: usize,
    pub max_dispatches_per_tick: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantLifecycleState {
    Active,
    Suspended,
    Restricted,
    Retiring,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantProvisioningSpec {
    pub tenant_id: TenantId,
    pub namespace: String,
    pub registry_partition: TenantRegistryPartition,
    pub default_queue_isolation: TenantQueueIsolationPolicy,
    pub default_policy_bundle: TenantPolicyBundleRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantIsolationConformanceReport {
    pub api_isolated: bool,
    pub scheduler_isolated: bool,
    pub artifact_isolated: bool,
    pub metrics_isolated: bool,
    pub lineage_isolated: bool,
    pub violations: Vec<String>,
}

pub fn resolve_tenant_overlay(
    global_defaults: &BTreeMap<String, String>,
    overlay: &TenantConfigOverlay,
) -> BTreeMap<String, String> {
    let mut merged = global_defaults.clone();
    for (k, v) in &overlay.values {
        merged.insert(k.clone(), v.clone());
    }
    for (k, v) in &overlay.overrides {
        merged.insert(k.clone(), v.clone());
    }
    merged
}

pub fn compose_tenant_run_id(tenant_id: &TenantId, run_local_id: &str) -> String {
    format!("{}::{}", tenant_id.0, run_local_id)
}

pub fn tenant_index_key(tenant_id: &TenantId, category: &str, id: &str) -> String {
    format!("{}/{}/{}", tenant_id.0, category, id)
}

pub fn check_scheduler_admission(
    queued_runs: usize,
    pending_dispatches: usize,
    policy: &TenantSchedulerAdmission,
) -> bool {
    queued_runs <= policy.max_enqueued_runs && pending_dispatches <= policy.max_dispatches_per_tick
}

pub fn enforce_tenant_plugin_allowlist(
    plugin_name: &str,
    allowlist: &TenantPluginAllowlist,
) -> bool {
    allowlist.allowed_plugins.iter().any(|p| p == plugin_name)
}

pub fn scope_lineage_query(
    requested_artifact_ids: &[String],
    scope: &TenantLineageScope,
) -> Vec<String> {
    let allowed = scope
        .allowed_artifact_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    requested_artifact_ids
        .iter()
        .filter(|id| allowed.contains(*id))
        .cloned()
        .collect()
}

pub fn tenant_provisioning_bootstrap(spec: &TenantProvisioningSpec) -> Vec<String> {
    vec![
        format!("create namespace {}", spec.namespace),
        format!(
            "initialize registry partition {}",
            spec.registry_partition.storage_partition
        ),
        format!(
            "configure queue isolation {}",
            spec.default_queue_isolation.queue_names.join(",")
        ),
        format!(
            "attach policy bundle {}:{}",
            spec.default_policy_bundle.policy_bundle_id,
            spec.default_policy_bundle.policy_bundle_version
        ),
    ]
}

pub fn validate_tenant_isolation(
    requested_tenant: &TenantId,
    api_tenant: &TenantId,
    scheduler_tenant: &TenantId,
    artifact_tenant: &TenantId,
    metrics_tenant: &TenantId,
    lineage_tenant: &TenantId,
) -> TenantIsolationConformanceReport {
    let api_isolated = requested_tenant == api_tenant;
    let scheduler_isolated = requested_tenant == scheduler_tenant;
    let artifact_isolated = requested_tenant == artifact_tenant;
    let metrics_isolated = requested_tenant == metrics_tenant;
    let lineage_isolated = requested_tenant == lineage_tenant;
    let mut violations = Vec::new();
    if !api_isolated {
        violations.push("api tenant scope mismatch".to_string());
    }
    if !scheduler_isolated {
        violations.push("scheduler tenant scope mismatch".to_string());
    }
    if !artifact_isolated {
        violations.push("artifact tenant scope mismatch".to_string());
    }
    if !metrics_isolated {
        violations.push("metrics tenant scope mismatch".to_string());
    }
    if !lineage_isolated {
        violations.push("lineage tenant scope mismatch".to_string());
    }
    TenantIsolationConformanceReport {
        api_isolated,
        scheduler_isolated,
        artifact_isolated,
        metrics_isolated,
        lineage_isolated,
        violations,
    }
}
