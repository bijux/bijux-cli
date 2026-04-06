use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: usize,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListFilter {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedResource {
    pub resource_version: u64,
    pub etag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagResource {
    pub dag_id: String,
    pub logical_name: String,
    pub owner: String,
    pub tags: Vec<String>,
    pub version: VersionedResource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagVersionResource {
    pub dag_id: String,
    pub version_id: String,
    pub status: String,
    pub compatibility_line: String,
    pub version: VersionedResource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunResource {
    pub run_id: String,
    pub dag_id: String,
    pub dag_version_id: String,
    pub status: String,
    pub submitted_unix_ms: u128,
    pub version: VersionedResource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAttemptResource {
    pub run_id: String,
    pub node_id: String,
    pub attempt: u32,
    pub status: String,
    pub started_unix_ms: u128,
    pub finished_unix_ms: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactResource {
    pub artifact_id: String,
    pub run_id: String,
    pub producer_node_id: String,
    pub schema_name: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleResource {
    pub schedule_id: String,
    pub dag_id: String,
    pub trigger_kind: String,
    pub queue: String,
    pub suspended: bool,
    pub version: VersionedResource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueResource {
    pub queue_id: String,
    pub tenant: Option<String>,
    pub priority_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyResource {
    pub policy_id: String,
    pub domain: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEventResource {
    pub audit_id: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryOperation {
    Publish,
    Validate,
    Activate,
    Deprecate,
    Retire,
    Inspect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunControlApiOperation {
    Submit,
    Cancel,
    Pause,
    Resume,
    Retry,
    Replay,
    Verify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactApiOperation {
    Inspect,
    Export,
    Verify,
    Lineage,
    RetentionAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleApiOperation {
    Create,
    Update,
    Suspend,
    Preview,
    Audit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedApiRequest {
    pub api_version: ApiVersion,
    pub operation: String,
    pub request_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedApiResponse {
    pub api_version: ApiVersion,
    pub accepted: bool,
    pub status: String,
    pub response_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthenticationPrincipal {
    CliUser { subject: String },
    ServiceAccount { service: String },
    WorkerIdentity { worker_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthContext {
    pub principal: AuthenticationPrincipal,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationRule {
    pub resource_prefix: String,
    pub allowed_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentScopedConfiguration {
    pub environment: String,
    pub values: BTreeMap<String, String>,
    pub overlays: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSubscription {
    pub subscription_id: String,
    pub topic: String,
    pub endpoint: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiCompatibilityRule {
    pub min_supported_major: u16,
    pub max_supported_major: u16,
    pub supports_minor_additive_fields: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientSdkShape {
    pub sdk_name: String,
    pub operations: Vec<String>,
    pub typed_models: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceArchitectureNote {
    pub api_boundary: String,
    pub scheduler_boundary: String,
    pub registry_boundary: String,
    pub executor_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPlaneMvpDefinition {
    pub includes_registry: bool,
    pub includes_run_control: bool,
    pub includes_schedule_management: bool,
    pub includes_audit_log: bool,
    pub excludes_distributed_orchestration: bool,
}

pub fn paginate<T: Clone>(items: &[T], pagination: &Pagination) -> Page<T> {
    let start = pagination
        .cursor
        .as_ref()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .min(items.len());
    let end = (start + pagination.limit.max(1)).min(items.len());
    let next_cursor = if end < items.len() { Some(end.to_string()) } else { None };
    Page { items: items[start..end].to_vec(), next_cursor }
}

pub fn filter_resources<T>(
    items: Vec<T>,
    filter: &ListFilter,
    value_of: impl Fn(&T, &str) -> Option<String>,
) -> Vec<T> {
    items
        .into_iter()
        .filter(|item| value_of(item, &filter.field).as_deref() == Some(filter.value.as_str()))
        .collect()
}

pub fn authorize(auth: &AuthContext, action: &str, rules: &[AuthorizationRule]) -> bool {
    rules.iter().any(|rule| {
        auth.scopes.iter().any(|scope| scope.starts_with(&rule.resource_prefix))
            && rule.allowed_actions.iter().any(|a| a == action)
    })
}

pub fn check_api_compatibility(version: &ApiVersion, rule: &ApiCompatibilityRule) -> bool {
    version.major >= rule.min_supported_major && version.major <= rule.max_supported_major
}
