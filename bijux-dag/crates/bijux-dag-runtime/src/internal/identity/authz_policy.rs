use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectKind {
    User,
    ServiceAccount,
    Worker,
    Scheduler,
    Automation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectIdentity {
    pub subject_id: String,
    pub kind: SubjectKind,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    Read,
    Write,
    Execute,
    Approve,
    Manage,
    Administer,
    Audit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub kind: ActionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Dag,
    DagVersion,
    Run,
    Node,
    Artifact,
    Schedule,
    Queue,
    Policy,
    Tenant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRef {
    pub kind: ResourceKind,
    pub id: String,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceScope {
    Global,
    Tenant { tenant_id: String },
    Dag { tenant_id: String, dag_id: String },
    Run { tenant_id: String, run_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionType {
    Allow,
    Deny,
    Conditional,
    Delegated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecisionRecord {
    pub decision: DecisionType,
    pub reason: String,
    pub policy_bundle_id: String,
    pub policy_bundle_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvaluationTrace {
    pub request_id: String,
    pub evaluated_rules: Vec<String>,
    pub matched_rules: Vec<String>,
    pub decision: DecisionType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvaluationRequest {
    pub request_id: String,
    pub subject: SubjectIdentity,
    pub action: Action,
    pub resource: ResourceRef,
    pub scope: ResourceScope,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub decision_record: PolicyDecisionRecord,
    pub trace: PolicyEvaluationTrace,
}

pub trait PolicyEvaluationEngine {
    fn evaluate(&self, request: &PolicyEvaluationRequest) -> PolicyEvaluationResult;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltInRole {
    Viewer,
    Operator,
    Developer,
    Releaser,
    TenantAdmin,
    PlatformAdmin,
    Auditor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub role: BuiltInRole,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomRoleDefinition {
    pub role_name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentAuthorizationRule {
    pub environment: String,
    pub denied_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionBoundary {
    pub run_control_permissions: Vec<String>,
    pub dag_publication_permissions: Vec<String>,
    pub artifact_access_permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensitiveControlPermissions {
    pub replay_allowed: bool,
    pub export_allowed: bool,
    pub promotion_allowed: bool,
    pub retention_override_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityPermissionProfile {
    pub scheduler_permissions: Vec<String>,
    pub worker_permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecisionCacheEntry {
    pub cache_key: String,
    pub decision: DecisionType,
    pub policy_bundle_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecisionCache {
    pub entries: Vec<PolicyDecisionCacheEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDryRunResult {
    pub would_allow: bool,
    pub reason: String,
    pub evaluated_policy_bundle_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationAcceptanceReport {
    pub least_privilege_holds: bool,
    pub denied_without_permission: bool,
    pub no_cross_tenant_escalation: bool,
    pub failures: Vec<String>,
}

pub fn builtin_role_definitions() -> Vec<RoleDefinition> {
    vec![
        RoleDefinition {
            role: BuiltInRole::Viewer,
            permissions: vec!["dag.read".to_string(), "run.read".to_string()],
        },
        RoleDefinition {
            role: BuiltInRole::Operator,
            permissions: vec![
                "run.read".to_string(),
                "run.cancel".to_string(),
                "run.pause".to_string(),
            ],
        },
        RoleDefinition {
            role: BuiltInRole::Developer,
            permissions: vec![
                "dag.read".to_string(),
                "dag.validate".to_string(),
                "run.submit".to_string(),
            ],
        },
        RoleDefinition {
            role: BuiltInRole::Releaser,
            permissions: vec!["dag.activate".to_string(), "artifact.promote".to_string()],
        },
        RoleDefinition {
            role: BuiltInRole::TenantAdmin,
            permissions: vec!["tenant.manage".to_string(), "policy.manage".to_string()],
        },
        RoleDefinition {
            role: BuiltInRole::PlatformAdmin,
            permissions: vec!["platform.administer".to_string()],
        },
        RoleDefinition {
            role: BuiltInRole::Auditor,
            permissions: vec!["audit.read".to_string(), "policy.trace.read".to_string()],
        },
    ]
}

pub fn validate_custom_role(role: &CustomRoleDefinition) -> Result<(), String> {
    if role.role_name.trim().is_empty() {
        return Err("custom role name must not be empty".to_string());
    }
    if role.permissions.is_empty() {
        return Err("custom role must include at least one permission".to_string());
    }
    let unsupported_combo = role.permissions.iter().any(|p| p == "platform.administer")
        && role.permissions.iter().any(|p| p == "tenant.manage");
    if unsupported_combo {
        return Err(
            "custom role cannot combine platform.administer and tenant.manage in one role"
                .to_string(),
        );
    }
    Ok(())
}

pub fn is_action_allowed_in_environment(
    action: &str,
    environment: &str,
    rules: &[EnvironmentAuthorizationRule],
) -> bool {
    let denied: BTreeSet<_> = rules
        .iter()
        .filter(|r| r.environment == environment)
        .flat_map(|r| r.denied_actions.iter().cloned())
        .collect();
    !denied.contains(action)
}

pub fn has_permission(permission: &str, permissions: &[String]) -> bool {
    permissions.iter().any(|p| p == permission)
}

pub fn decision_cache_key(
    request: &PolicyEvaluationRequest,
    policy_bundle_version: &str,
) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        request.subject.subject_id,
        request.action.name,
        request.resource.id,
        request.environment,
        policy_bundle_version
    )
}

pub fn invalidate_decision_cache(cache: &mut PolicyDecisionCache, policy_bundle_version: &str) {
    cache
        .entries
        .retain(|entry| entry.policy_bundle_version == policy_bundle_version);
}

pub fn evaluate_dry_run(
    request: &PolicyEvaluationRequest,
    allowed_permissions: &[String],
    policy_bundle_version: &str,
) -> PolicyDryRunResult {
    let permission_match = has_permission(&request.action.name, allowed_permissions);
    PolicyDryRunResult {
        would_allow: permission_match,
        reason: if permission_match {
            "dry-run allow: action is present in granted permissions".to_string()
        } else {
            "dry-run deny: action is not present in granted permissions".to_string()
        },
        evaluated_policy_bundle_version: policy_bundle_version.to_string(),
    }
}

pub fn evaluate_authorization_acceptance(
    decisions: &[(String, DecisionType)],
    cross_tenant_denials: &[bool],
) -> AuthorizationAcceptanceReport {
    let denied_without_permission = decisions.iter().any(|(action, decision)| {
        action.contains("admin") && matches!(decision, DecisionType::Deny)
    });
    let least_privilege_holds = decisions.iter().all(|(action, decision)| {
        !(action.contains("admin") && matches!(decision, DecisionType::Allow))
    });
    let no_cross_tenant_escalation = cross_tenant_denials.iter().all(|d| *d);
    let mut failures = Vec::new();
    if !least_privilege_holds {
        failures.push("least-privilege boundary violated".to_string());
    }
    if !no_cross_tenant_escalation {
        failures.push("cross-tenant escalation was allowed".to_string());
    }
    AuthorizationAcceptanceReport {
        least_privilege_holds,
        denied_without_permission,
        no_cross_tenant_escalation,
        failures,
    }
}

pub fn role_catalog_by_name() -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();
    for role in builtin_role_definitions() {
        map.insert(format!("{:?}", role.role), role.permissions);
    }
    map
}
