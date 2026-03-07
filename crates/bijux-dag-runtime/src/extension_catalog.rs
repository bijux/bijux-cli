use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginBoundaryKind {
    TaskAdapter,
    ExecutorBackend,
    ArtifactStore,
    ObservabilitySink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRange {
    pub min_contract_version: String,
    pub max_contract_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub boundary: PluginBoundaryKind,
    pub capabilities: Vec<String>,
    pub policy_requirements: Vec<String>,
    pub compatibility: CapabilityRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginLoadingMode {
    StaticLinking,
    DynamicLoadingDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionRegistration {
    pub plugin_name: String,
    pub plugin_version: String,
    pub boundary: PluginBoundaryKind,
    pub registered_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDescriptor {
    pub plugin_name: String,
    pub plugin_version: String,
    pub boundary: PluginBoundaryKind,
    pub contract_version: String,
    pub capabilities: Vec<String>,
    pub trust_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionCompatibilityIssue {
    pub plugin_name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionStabilityLevel {
    InternalHook,
    Experimental,
    Stable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionPointStatus {
    pub extension_point: String,
    pub stability: ExtensionStabilityLevel,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternalHookPromotionChecklist {
    pub hook_name: String,
    pub has_contract_doc: bool,
    pub has_versioning_policy: bool,
    pub has_negative_tests: bool,
    pub has_failure_isolation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginTrustPolicy {
    pub require_signature: bool,
    pub allowlisted_publishers: Vec<String>,
    pub allowed_environments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginIsolationPolicy {
    pub deny_undeclared_effects: bool,
    pub require_deterministic_mode: bool,
    pub enforce_resource_caps: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginConformanceSuiteResult {
    pub plugin_name: String,
    pub boundary: PluginBoundaryKind,
    pub passed: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginLifecycleState {
    Develop,
    Register,
    Validate,
    Release,
    Deprecate,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficialPluginPolicy {
    pub requires_core_team_review: bool,
    pub requires_security_assessment: bool,
    pub release_channel: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDiscoveryRecord {
    pub plugin_name: String,
    pub boundary: PluginBoundaryKind,
    pub capabilities: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DslExtensionPoint {
    pub node_family: String,
    pub contract_name: String,
    pub compile_time_validated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeGenerationHook {
    pub hook_name: String,
    pub produces: Vec<String>,
    pub consumes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionRoadmap {
    pub core_surface: Vec<String>,
    pub pluggable_surface: Vec<String>,
    pub intentionally_unsupported: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformMaturityScorecard {
    pub engine_ready: u8,
    pub scheduler_ready: u8,
    pub artifacts_ready: u8,
    pub observability_ready: u8,
    pub api_ready: u8,
    pub infrastructure_ready: u8,
    pub extension_ready: u8,
}

pub fn negotiate_plugin_version(range: &CapabilityRange, requested_contract_version: &str) -> bool {
    requested_contract_version >= range.min_contract_version.as_str()
        && requested_contract_version <= range.max_contract_version.as_str()
}

pub fn validate_plugin_conformance(
    metadata: &PluginMetadata,
    trust: &PluginTrustPolicy,
    isolation: &PluginIsolationPolicy,
) -> PluginConformanceSuiteResult {
    let mut failures = Vec::new();
    if metadata.name.trim().is_empty() {
        failures.push("plugin name must not be empty".to_string());
    }
    if metadata.version.trim().is_empty() {
        failures.push("plugin version must not be empty".to_string());
    }
    if metadata.capabilities.is_empty() {
        failures.push("plugin must declare at least one capability".to_string());
    }
    if trust.require_signature && trust.allowlisted_publishers.is_empty() {
        failures.push("signature-required plugins need allowlisted publishers".to_string());
    }
    if isolation.require_deterministic_mode && !metadata.policy_requirements.iter().any(|r| r == "deterministic") {
        failures.push("plugin must require deterministic policy".to_string());
    }
    PluginConformanceSuiteResult {
        plugin_name: metadata.name.clone(),
        boundary: metadata.boundary.clone(),
        passed: failures.is_empty(),
        failures,
    }
}

pub fn extension_discovery_inventory(registrations: &[ExtensionRegistration]) -> Vec<ExtensionDiscoveryRecord> {
    let mut out = registrations
        .iter()
        .map(|r| ExtensionDiscoveryRecord {
            plugin_name: r.plugin_name.clone(),
            boundary: r.boundary.clone(),
            capabilities: Vec::new(),
            active: true,
        })
        .collect::<Vec<_>>();
    out.sort_by(|a, b| a.plugin_name.cmp(&b.plugin_name));
    out
}

pub fn compute_platform_maturity(scores: &[u8]) -> u8 {
    if scores.is_empty() {
        return 0;
    }
    let total: u32 = scores.iter().map(|v| *v as u32).sum();
    (total / scores.len() as u32) as u8
}

pub fn validate_extension_descriptor(descriptor: &ExtensionDescriptor) -> Result<(), String> {
    if descriptor.plugin_name.trim().is_empty() {
        return Err("plugin_name must not be empty".to_string());
    }
    if descriptor.plugin_version.trim().is_empty() {
        return Err("plugin_version must not be empty".to_string());
    }
    if !descriptor.contract_version.starts_with('v') {
        return Err("contract_version must use v-prefixed format".to_string());
    }
    if descriptor.capabilities.is_empty() {
        return Err("descriptor must declare capabilities".to_string());
    }
    Ok(())
}

pub fn register_extension(
    registry: &mut BTreeMap<String, ExtensionDescriptor>,
    descriptor: ExtensionDescriptor,
) -> Result<(), String> {
    validate_extension_descriptor(&descriptor)?;
    if registry.contains_key(&descriptor.plugin_name) {
        return Err(format!(
            "extension registration conflict for {}",
            descriptor.plugin_name
        ));
    }
    registry.insert(descriptor.plugin_name.clone(), descriptor);
    Ok(())
}

pub fn detect_extension_compatibility_issues(
    descriptors: &[ExtensionDescriptor],
    supported_contract_versions: &BTreeSet<String>,
    required_capabilities: &BTreeSet<String>,
) -> Vec<ExtensionCompatibilityIssue> {
    let mut issues = Vec::new();
    for descriptor in descriptors {
        if !supported_contract_versions.contains(&descriptor.contract_version) {
            issues.push(ExtensionCompatibilityIssue {
                plugin_name: descriptor.plugin_name.clone(),
                reason: format!(
                    "unsupported contract version {}",
                    descriptor.contract_version
                ),
            });
        }
        for capability in required_capabilities {
            if !descriptor.capabilities.iter().any(|c| c == capability) {
                issues.push(ExtensionCompatibilityIssue {
                    plugin_name: descriptor.plugin_name.clone(),
                    reason: format!("missing required capability {}", capability),
                });
            }
        }
    }
    issues
}

pub fn extension_point_status_report() -> Vec<ExtensionPointStatus> {
    vec![
        ExtensionPointStatus {
            extension_point: "task_adapter".to_string(),
            stability: ExtensionStabilityLevel::Stable,
            owner: "runtime".to_string(),
        },
        ExtensionPointStatus {
            extension_point: "executor_backend".to_string(),
            stability: ExtensionStabilityLevel::Experimental,
            owner: "runtime".to_string(),
        },
        ExtensionPointStatus {
            extension_point: "validation_hook".to_string(),
            stability: ExtensionStabilityLevel::InternalHook,
            owner: "core".to_string(),
        },
    ]
}

pub fn internal_hook_ready_for_promotion(check: &InternalHookPromotionChecklist) -> bool {
    check.has_contract_doc
        && check.has_versioning_policy
        && check.has_negative_tests
        && check.has_failure_isolation
}

pub fn extension_failure_isolated(failure_scope: &str, engine_crashed: bool) -> bool {
    failure_scope == "extension_only" && !engine_crashed
}
