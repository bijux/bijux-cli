#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::routing::{PluginLifecycleState, PluginManifestV1};
use serde::{Deserialize, Serialize};

/// Runtime-facing plugin record persisted in registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRecord {
    /// Plugin manifest.
    pub manifest: PluginManifestV1,
    /// Plugin lifecycle state.
    pub state: PluginLifecycleState,
    /// Source artifact reference.
    pub source: String,
    /// Plugin trust level.
    pub trust_level: PluginTrustLevel,
    /// SHA-256 digest of raw manifest text.
    pub manifest_checksum_sha256: String,
}

/// Trust-level model for plugin provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginTrustLevel {
    /// Official core-distributed plugin.
    Core,
    /// Verified plugin provenance.
    Verified,
    /// Community plugin.
    Community,
    /// Unknown provenance.
    Unknown,
}

/// Durable plugin registry file model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRegistry {
    /// Registry schema version.
    pub version: String,
    /// Installed plugins by namespace.
    pub plugins: BTreeMap<String, PluginRecord>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self {
            version: super::constants::REGISTRY_VERSION.to_string(),
            plugins: BTreeMap::new(),
        }
    }
}

/// Plugin discovery cache.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PluginDiscoveryCache {
    /// Discovery root path.
    pub root: PathBuf,
    /// Last known manifests by namespace.
    pub manifests: BTreeMap<String, PathBuf>,
    /// Last update timestamp in unix millis.
    pub last_updated_millis: u128,
}

/// Load ordering entry for diagnostics and deterministic execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLoadEntry {
    /// Namespace.
    pub namespace: String,
    /// Current state.
    pub state: PluginLifecycleState,
}

/// Load diagnostics item for plugins that cannot be executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLoadDiagnostic {
    /// Namespace.
    pub namespace: String,
    /// Severity for display and automation.
    pub severity: String,
    /// Human-readable message.
    pub message: String,
}

/// Install request model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPluginRequest {
    /// Raw manifest text.
    pub manifest_text: String,
    /// Provenance source string.
    pub source: String,
    /// Assigned trust level.
    pub trust_level: PluginTrustLevel,
}

/// Validate manifest and represent normalized validation output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPlugin {
    /// Valid manifest.
    pub manifest: PluginManifestV1,
    /// Initial lifecycle state after validation.
    pub state: PluginLifecycleState,
}

/// Operational status summary for plugin subsystem diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDoctorReport {
    /// Number of installed plugins.
    pub installed: usize,
    /// Namespaces in broken state.
    pub broken: Vec<String>,
    /// Namespaces in incompatible state.
    pub incompatible: Vec<String>,
}

/// Plugin origin metadata for route introspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginOriginMetadata {
    /// Namespace.
    pub namespace: String,
    /// Source reference.
    pub source: String,
    /// Trust level.
    pub trust_level: PluginTrustLevel,
}
