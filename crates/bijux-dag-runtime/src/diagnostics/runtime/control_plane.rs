use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DagVersionStatus {
    Draft,
    Validated,
    Active,
    Deprecated,
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DagVersionRecord {
    pub version_id: String,
    pub compatibility_line: String,
    pub status: DagVersionStatus,
    pub created_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DagRegistryEntry {
    pub dag_name: String,
    pub owner: String,
    pub tags: Vec<String>,
    pub versions: Vec<DagVersionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DagRegistry {
    pub entries: BTreeMap<String, DagRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DagVersionSelectionPolicy {
    RunLatest,
    RunPinned { version_id: String },
    RunCompatible { compatibility_line: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityDecision {
    Selected { version_id: String, reason: String },
    Rejected { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationRequest {
    pub dag_name: String,
    pub version_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationResponse {
    pub valid: bool,
    pub diagnostics: Vec<String>,
}

pub trait ValidationService {
    fn validate(&self, request: ValidationRequest) -> ValidationResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyDomain {
    Repository,
    Runtime,
    Organization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyBundle {
    pub bundle_id: String,
    pub version: String,
    pub domain: PolicyDomain,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: String,
}

pub trait PolicyEngine {
    fn evaluate(&self, bundle: &PolicyBundle, action: &str, resource: &str) -> PolicyDecision;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationSubject {
    pub subject_id: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationRequest {
    pub subject: AuthorizationSubject,
    pub action: String,
    pub resource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnvironmentMode {
    Local,
    Ci,
    Staging,
    Production,
    Airgapped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentConfiguration {
    pub mode: EnvironmentMode,
    pub parent: Option<String>,
    pub values: BTreeMap<String, String>,
    pub overrides: BTreeMap<String, String>,
}

pub fn resolve_environment_values(
    current: &EnvironmentConfiguration,
    parent: Option<&EnvironmentConfiguration>,
) -> BTreeMap<String, String> {
    let mut merged = BTreeMap::new();
    if let Some(parent_cfg) = parent {
        for (key, value) in &parent_cfg.values {
            merged.insert(key.clone(), value.clone());
        }
        for (key, value) in &parent_cfg.overrides {
            merged.insert(key.clone(), value.clone());
        }
    }
    for (key, value) in &current.values {
        merged.insert(key.clone(), value.clone());
    }
    for (key, value) in &current.overrides {
        merged.insert(key.clone(), value.clone());
    }
    merged
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunControlOperation {
    Submit,
    Cancel,
    Pause,
    Resume,
    Retry,
    Replay,
    Export,
    Verify,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypedControlPlaneRequest {
    pub operation: RunControlOperation,
    pub dag_name: String,
    pub run_id: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypedControlPlaneResponse {
    pub accepted: bool,
    pub message: String,
    pub details: serde_json::Value,
}

pub trait DagRegistryStore {
    fn load_registry(&self) -> Result<DagRegistry, String>;
    fn save_registry(&self, registry: &DagRegistry) -> Result<(), String>;
}

pub fn register_dag_version(
    registry: &mut DagRegistry,
    dag_name: &str,
    owner: &str,
    tags: Vec<String>,
    record: DagVersionRecord,
) -> Result<(), String> {
    let entry = registry
        .entries
        .entry(dag_name.to_string())
        .or_insert_with(|| DagRegistryEntry {
            dag_name: dag_name.to_string(),
            owner: owner.to_string(),
            tags,
            versions: Vec::new(),
        });
    if entry
        .versions
        .iter()
        .any(|v| v.version_id == record.version_id)
    {
        return Err(format!(
            "dag '{}' already contains version '{}'",
            dag_name, record.version_id
        ));
    }
    entry.versions.push(record);
    entry
        .versions
        .sort_by(|a, b| a.version_id.cmp(&b.version_id));
    Ok(())
}

pub fn select_dag_version(
    registry: &DagRegistry,
    dag_name: &str,
    policy: &DagVersionSelectionPolicy,
) -> CompatibilityDecision {
    let Some(entry) = registry.entries.get(dag_name) else {
        return CompatibilityDecision::Rejected {
            reason: format!("dag '{}' not found", dag_name),
        };
    };
    match policy {
        DagVersionSelectionPolicy::RunLatest => entry
            .versions
            .iter()
            .filter(|v| {
                matches!(
                    v.status,
                    DagVersionStatus::Validated | DagVersionStatus::Active
                )
            })
            .max_by(|a, b| a.version_id.cmp(&b.version_id))
            .map(|v| CompatibilityDecision::Selected {
                version_id: v.version_id.clone(),
                reason: "selected latest validated or active version".to_string(),
            })
            .unwrap_or(CompatibilityDecision::Rejected {
                reason: "no validated or active versions available".to_string(),
            }),
        DagVersionSelectionPolicy::RunPinned { version_id } => {
            if entry.versions.iter().any(|v| &v.version_id == version_id) {
                CompatibilityDecision::Selected {
                    version_id: version_id.clone(),
                    reason: "selected pinned version".to_string(),
                }
            } else {
                CompatibilityDecision::Rejected {
                    reason: format!("pinned version '{}' was not found", version_id),
                }
            }
        }
        DagVersionSelectionPolicy::RunCompatible { compatibility_line } => entry
            .versions
            .iter()
            .filter(|v| &v.compatibility_line == compatibility_line)
            .filter(|v| !matches!(v.status, DagVersionStatus::Retired))
            .max_by(|a, b| a.version_id.cmp(&b.version_id))
            .map(|v| CompatibilityDecision::Selected {
                version_id: v.version_id.clone(),
                reason: format!(
                    "selected highest compatible version in '{}'",
                    compatibility_line
                ),
            })
            .unwrap_or(CompatibilityDecision::Rejected {
                reason: format!(
                    "no compatible versions found in compatibility line '{}'",
                    compatibility_line
                ),
            }),
    }
}
