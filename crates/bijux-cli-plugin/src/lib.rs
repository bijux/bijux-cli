#![forbid(unsafe_code)]
//! Plugin registration and lifecycle boundaries.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use bijux_cli_contracts::{
    CompatibilityRange, ContractMarker, Namespace, PluginKind, PluginLifecycleState,
    PluginManifestV1,
};
use bijux_cli_core::core_marker;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

const REGISTRY_VERSION: &str = "1";

/// Reserved namespaces that plugins cannot claim.
pub const RESERVED_NAMESPACES: &[&str] = &[
    "cli",
    "dev",
    "help",
    "version",
    "doctor",
    "repl",
    "plugins",
    "completion",
    "inspect",
];

/// Runtime-facing plugin record persisted in registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRecord {
    /// Plugin manifest.
    pub manifest: PluginManifestV1,
    /// Plugin lifecycle state.
    pub state: PluginLifecycleState,
    /// Source artifact reference.
    pub source: String,
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
        Self { version: REGISTRY_VERSION.to_string(), plugins: BTreeMap::new() }
    }
}

/// Plugin manifest parsing/validation/registry errors.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Manifest parse failed.
    #[error("plugin manifest parse failed: {0}")]
    ManifestParse(String),
    /// Missing or invalid required field.
    #[error("plugin manifest field invalid: {0}")]
    InvalidField(String),
    /// Namespace format is invalid.
    #[error("plugin namespace is invalid: {0}")]
    InvalidNamespace(String),
    /// Namespace is reserved.
    #[error("plugin namespace is reserved: {0}")]
    ReservedNamespace(String),
    /// Alias duplication detected.
    #[error("plugin manifest contains duplicate alias: {0}")]
    DuplicateAlias(String),
    /// Plugin compatibility does not include host version.
    #[error("plugin is incompatible with host version {host_version}")]
    IncompatibleVersion {
        /// Host version used for validation.
        host_version: String,
    },
    /// Plugin entrypoint is invalid for selected kind.
    #[error("plugin entrypoint is invalid for kind {kind:?}")]
    InvalidEntrypoint {
        /// Plugin kind.
        kind: PluginKind,
    },
    /// Plugin kind is not supported by current runtime.
    #[error("plugin kind is not supported in current runtime: {0:?}")]
    UnsupportedKind(PluginKind),
    /// Plugin namespace already exists in registry.
    #[error("plugin namespace already installed: {0}")]
    NamespaceConflict(String),
    /// Registry file is corrupted.
    #[error("plugin registry is corrupted")]
    RegistryCorrupted,
    /// Plugin not found by namespace.
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    /// I/O failure.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// JSON failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Install request model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPluginRequest {
    /// Raw manifest text.
    pub manifest_text: String,
    /// Provenance source string.
    pub source: String,
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

/// Build plugin marker chained from core state.
#[must_use]
pub fn plugin_marker() -> ContractMarker {
    let mut marker = core_marker();
    marker.namespace = format!("{}:plugin", marker.namespace);
    marker
}

/// Parse `PluginManifestV1` from JSON text.
pub fn parse_manifest_v1(text: &str) -> Result<PluginManifestV1, PluginError> {
    serde_json::from_str(text).map_err(|error| PluginError::ManifestParse(error.to_string()))
}

/// Validate plugin manifest against host compatibility and namespace rules.
pub fn validate_manifest(
    manifest: PluginManifestV1,
    host_version: &str,
    reserved_namespaces: &[&str],
) -> Result<ValidatedPlugin, PluginError> {
    validate_required_fields(&manifest)?;
    validate_namespace_format(&manifest.namespace)?;
    reject_reserved_namespace(&manifest.namespace, reserved_namespaces)?;
    validate_aliases(&manifest.aliases)?;
    validate_compatibility(&manifest.compatibility, host_version)?;
    validate_entrypoint_and_kind(&manifest)?;

    let state = if is_version_compatible(&manifest.compatibility, host_version)? {
        PluginLifecycleState::Validated
    } else {
        PluginLifecycleState::Incompatible
    };

    Ok(ValidatedPlugin { manifest, state })
}

fn validate_required_fields(manifest: &PluginManifestV1) -> Result<(), PluginError> {
    if manifest.name.trim().is_empty() {
        return Err(PluginError::InvalidField("name".to_string()));
    }
    if manifest.version.trim().is_empty() {
        return Err(PluginError::InvalidField("version".to_string()));
    }
    if manifest.schema_version.trim().is_empty() {
        return Err(PluginError::InvalidField("schema_version".to_string()));
    }
    if manifest.manifest_version.trim().is_empty() {
        return Err(PluginError::InvalidField("manifest_version".to_string()));
    }
    Ok(())
}

fn validate_namespace_format(namespace: &Namespace) -> Result<(), PluginError> {
    let raw = namespace.0.as_str();
    let bytes = raw.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return Err(PluginError::InvalidNamespace(raw.to_string()));
    }

    if !bytes
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
    {
        return Err(PluginError::InvalidNamespace(raw.to_string()));
    }

    if raw.contains("--") || raw.ends_with('-') {
        return Err(PluginError::InvalidNamespace(raw.to_string()));
    }

    Ok(())
}

fn reject_reserved_namespace(namespace: &Namespace, reserved: &[&str]) -> Result<(), PluginError> {
    if reserved.iter().any(|value| *value == namespace.0) {
        return Err(PluginError::ReservedNamespace(namespace.0.clone()));
    }
    Ok(())
}

fn validate_aliases(aliases: &[String]) -> Result<(), PluginError> {
    let mut seen = BTreeSet::new();
    for alias in aliases {
        if !seen.insert(alias.to_ascii_lowercase()) {
            return Err(PluginError::DuplicateAlias(alias.clone()));
        }
    }
    Ok(())
}

fn validate_compatibility(range: &CompatibilityRange, host_version: &str) -> Result<(), PluginError> {
    if !is_version_compatible(range, host_version)? {
        return Err(PluginError::IncompatibleVersion {
            host_version: host_version.to_string(),
        });
    }
    Ok(())
}

fn is_version_compatible(range: &CompatibilityRange, host_version: &str) -> Result<bool, PluginError> {
    let host = Version::parse(host_version)
        .map_err(|_| PluginError::InvalidField("host_version".to_string()))?;
    let min = Version::parse(&range.min_inclusive)
        .map_err(|_| PluginError::InvalidField("compatibility.min_inclusive".to_string()))?;
    if host < min {
        return Ok(false);
    }

    if let Some(max_exclusive) = &range.max_exclusive {
        let max = Version::parse(max_exclusive)
            .map_err(|_| PluginError::InvalidField("compatibility.max_exclusive".to_string()))?;
        if host >= max {
            return Ok(false);
        }
    }

    Ok(true)
}

fn validate_entrypoint_and_kind(manifest: &PluginManifestV1) -> Result<(), PluginError> {
    if manifest.entrypoint.trim().is_empty() {
        return Err(PluginError::InvalidEntrypoint { kind: manifest.kind });
    }

    match manifest.kind {
        PluginKind::Delegated | PluginKind::Python => {
            if !manifest.entrypoint.contains(':') && !manifest.entrypoint.contains('.') {
                return Err(PluginError::InvalidEntrypoint { kind: manifest.kind });
            }
        }
        PluginKind::ExternalExec => {
            if manifest.entrypoint.contains(':') {
                return Err(PluginError::InvalidEntrypoint { kind: manifest.kind });
            }
        }
        PluginKind::Native => return Err(PluginError::UnsupportedKind(PluginKind::Native)),
    }

    Ok(())
}

/// Load plugin registry from disk.
pub fn load_registry(path: &Path) -> Result<PluginRegistry, PluginError> {
    if !path.exists() {
        return Ok(PluginRegistry::default());
    }

    let text = fs::read_to_string(path)?;
    let parsed: PluginRegistry =
        serde_json::from_str(&text).map_err(|_| PluginError::RegistryCorrupted)?;
    if parsed.version != REGISTRY_VERSION {
        return Err(PluginError::RegistryCorrupted);
    }
    Ok(parsed)
}

/// Save plugin registry atomically.
pub fn save_registry(path: &Path, registry: &PluginRegistry) -> Result<(), PluginError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_vec_pretty(registry)?;
    let temporary = path.with_extension("tmp");

    {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(&data)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }

    fs::rename(temporary, path)?;
    Ok(())
}

/// Update plugin registry atomically.
pub fn update_registry<F>(path: &Path, mutator: F) -> Result<PluginRegistry, PluginError>
where
    F: FnOnce(&mut PluginRegistry) -> Result<(), PluginError>,
{
    let mut registry = load_registry(path)?;
    mutator(&mut registry)?;
    save_registry(path, &registry)?;
    Ok(registry)
}

/// Install plugin into registry from manifest text.
pub fn install_plugin(
    registry_path: &Path,
    request: InstallPluginRequest,
    host_version: &str,
) -> Result<PluginRecord, PluginError> {
    let manifest = parse_manifest_v1(&request.manifest_text)?;
    let validated = validate_manifest(manifest, host_version, RESERVED_NAMESPACES)?;

    let namespace = validated.manifest.namespace.0.clone();
    let source = request.source;
    let record = PluginRecord {
        manifest: validated.manifest,
        state: PluginLifecycleState::Installed,
        source,
    };

    update_registry(registry_path, |registry| {
        if registry.plugins.contains_key(&namespace) {
            return Err(PluginError::NamespaceConflict(namespace.clone()));
        }
        registry.plugins.insert(namespace.clone(), record.clone());
        Ok(())
    })?;

    Ok(record)
}

/// Remove plugin from registry.
pub fn uninstall_plugin(registry_path: &Path, namespace: &str) -> Result<(), PluginError> {
    update_registry(registry_path, |registry| {
        if registry.plugins.remove(namespace).is_none() {
            return Err(PluginError::PluginNotFound(namespace.to_string()));
        }
        Ok(())
    })?;
    Ok(())
}

/// Enable installed plugin.
pub fn enable_plugin(registry_path: &Path, namespace: &str) -> Result<PluginRecord, PluginError> {
    set_plugin_state(registry_path, namespace, PluginLifecycleState::Enabled)
}

/// Disable installed plugin.
pub fn disable_plugin(registry_path: &Path, namespace: &str) -> Result<PluginRecord, PluginError> {
    set_plugin_state(registry_path, namespace, PluginLifecycleState::Disabled)
}

fn set_plugin_state(
    registry_path: &Path,
    namespace: &str,
    state: PluginLifecycleState,
) -> Result<PluginRecord, PluginError> {
    let mut updated: Option<PluginRecord> = None;

    update_registry(registry_path, |registry| {
        let plugin =
            registry.plugins.get_mut(namespace).ok_or_else(|| PluginError::PluginNotFound(namespace.to_string()))?;
        plugin.state = state;
        updated = Some(plugin.clone());
        Ok(())
    })?;

    updated.ok_or_else(|| PluginError::PluginNotFound(namespace.to_string()))
}

/// Inspect plugin by namespace.
pub fn inspect_plugin(registry_path: &Path, namespace: &str) -> Result<PluginRecord, PluginError> {
    let registry = load_registry(registry_path)?;
    registry
        .plugins
        .get(namespace)
        .cloned()
        .ok_or_else(|| PluginError::PluginNotFound(namespace.to_string()))
}

/// List all plugins deterministically by namespace.
pub fn list_plugins(registry_path: &Path) -> Result<Vec<PluginRecord>, PluginError> {
    let registry = load_registry(registry_path)?;
    Ok(registry.plugins.into_values().collect())
}

/// Produce plugin health report.
pub fn plugin_doctor(registry_path: &Path) -> Result<PluginDoctorReport, PluginError> {
    let registry = load_registry(registry_path)?;

    let mut broken = Vec::new();
    let mut incompatible = Vec::new();

    for (namespace, plugin) in &registry.plugins {
        if plugin.state == PluginLifecycleState::Broken {
            broken.push(namespace.clone());
        }
        if plugin.state == PluginLifecycleState::Incompatible {
            incompatible.push(namespace.clone());
        }
    }

    Ok(PluginDoctorReport { installed: registry.plugins.len(), broken, incompatible })
}

/// Check plugin compatibility against host version without mutating registry.
pub fn compatibility_check(
    manifest: &PluginManifestV1,
    host_version: &str,
) -> Result<bool, PluginError> {
    validate_version_req(host_version)?;
    is_version_compatible(&manifest.compatibility, host_version)
}

fn validate_version_req(host_version: &str) -> Result<(), PluginError> {
    let requirement = format!("={host_version}");
    VersionReq::parse(&requirement)
        .map(|_| ())
        .map_err(|_| PluginError::InvalidField("host_version".to_string()))
}

/// Parse registry path from the plugin directory.
#[must_use]
pub fn registry_path_from_plugins_dir(plugins_dir: &Path) -> PathBuf {
    plugins_dir.join("registry.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest(namespace: &str) -> String {
        format!(
            r#"{{
  "name": "{namespace}",
  "version": "1.0.0",
  "schema_version": "1",
  "manifest_version": "1",
  "compatibility": {{ "min_inclusive": "0.1.0", "max_exclusive": "2.0.0" }},
  "namespace": "{namespace}",
  "kind": "delegated",
  "aliases": ["{namespace}-alias"],
  "entrypoint": "{namespace}.plugin:run",
  "capabilities": [{{"name": "exec", "version": "1"}}]
}}"#
        )
    }

    #[test]
    fn validates_and_rejects_reserved_namespace() {
        let manifest = parse_manifest_v1(&sample_manifest("community")).expect("parse");
        let validated = validate_manifest(manifest, "0.1.0", RESERVED_NAMESPACES).expect("valid");
        assert_eq!(validated.state, PluginLifecycleState::Validated);

        let reserved_manifest = parse_manifest_v1(&sample_manifest("cli")).expect("parse");
        let error = validate_manifest(reserved_manifest, "0.1.0", RESERVED_NAMESPACES)
            .expect_err("reserved namespace must fail");
        assert!(matches!(error, PluginError::ReservedNamespace(_)));
    }
}
