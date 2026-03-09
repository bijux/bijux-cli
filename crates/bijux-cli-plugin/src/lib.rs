#![forbid(unsafe_code)]
//! Plugin registration and lifecycle boundaries.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_contracts::{
    CompatibilityRange, ContractMarker, Namespace, PluginCapability, PluginKind,
    PluginLifecycleState, PluginManifestV1,
};
use bijux_cli_core::core_marker;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

/// Reserved namespaces currently owned by bijux-cli core command graph.
pub const CORE_NAMESPACES: &[&str] = &["cli", "dev"];

/// Reserved namespaces for future official Bijux product mounts.
pub const FUTURE_PRODUCT_NAMESPACES: &[&str] = &["atlas", "cloud", "ops", "security"];

/// Runtime-facing plugin record persisted in registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRecord {
    /// Plugin manifest.
    pub manifest: PluginManifestV1,
    /// Plugin lifecycle state.
    pub state: PluginLifecycleState,
    /// Source artifact reference.
    pub source: String,
    /// SHA-256 digest of raw manifest text.
    pub manifest_checksum_sha256: String,
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
    /// Namespace collides with core namespace.
    #[error("plugin namespace collides with core namespace: {0}")]
    CoreNamespaceConflict(String),
    /// Namespace collides with future official product namespace.
    #[error("plugin namespace collides with reserved product namespace: {0}")]
    FutureNamespaceConflict(String),
    /// Alias duplication detected in single manifest.
    #[error("plugin manifest contains duplicate alias: {0}")]
    DuplicateAlias(String),
    /// Alias collides with an already installed plugin alias.
    #[error("plugin alias conflicts with installed plugin: {0}")]
    AliasConflict(String),
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
    /// Delegated plugin execution denied due missing capability.
    #[error("plugin is missing required capability: {0}")]
    MissingCapability(String),
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
    reject_core_namespace(&manifest.namespace)?;
    reject_future_product_namespace(&manifest.namespace)?;
    validate_aliases(&manifest.aliases)?;
    validate_compatibility(&manifest.compatibility, host_version)?;
    validate_entrypoint_and_kind(&manifest)?;

    Ok(ValidatedPlugin { manifest, state: PluginLifecycleState::Validated })
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

fn reject_core_namespace(namespace: &Namespace) -> Result<(), PluginError> {
    if CORE_NAMESPACES.iter().any(|value| *value == namespace.0) {
        return Err(PluginError::CoreNamespaceConflict(namespace.0.clone()));
    }
    Ok(())
}

fn reject_future_product_namespace(namespace: &Namespace) -> Result<(), PluginError> {
    if FUTURE_PRODUCT_NAMESPACES.iter().any(|value| *value == namespace.0) {
        return Err(PluginError::FutureNamespaceConflict(namespace.0.clone()));
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

fn checksum_sha256(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{digest:x}")
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

fn backup_registry(path: &Path) -> Result<Option<PathBuf>, PluginError> {
    if !path.exists() {
        return Ok(None);
    }
    let backup = path.with_extension("bak");
    fs::copy(path, &backup)?;
    Ok(Some(backup))
}

fn restore_registry(path: &Path, backup: Option<PathBuf>) -> Result<(), PluginError> {
    if let Some(backup_path) = backup {
        fs::rename(backup_path, path)?;
    }
    Ok(())
}

fn cleanup_backup(backup: Option<PathBuf>) {
    if let Some(path) = backup {
        let _ = fs::remove_file(path);
    }
}

/// Update plugin registry atomically with rollback support.
pub fn update_registry<F>(path: &Path, mutator: F) -> Result<PluginRegistry, PluginError>
where
    F: FnOnce(&mut PluginRegistry) -> Result<(), PluginError>,
{
    let backup = backup_registry(path)?;
    let mut registry = load_registry(path)?;

    if let Err(error) = mutator(&mut registry) {
        restore_registry(path, backup)?;
        return Err(error);
    }

    if let Err(error) = save_registry(path, &registry) {
        restore_registry(path, backup)?;
        return Err(error);
    }

    cleanup_backup(backup);
    Ok(registry)
}

/// Install plugin into registry from manifest text.
pub fn install_plugin(
    registry_path: &Path,
    request: InstallPluginRequest,
    host_version: &str,
) -> Result<PluginRecord, PluginError> {
    let manifest_checksum_sha256 = checksum_sha256(&request.manifest_text);
    let manifest = parse_manifest_v1(&request.manifest_text)?;
    let validated = validate_manifest(manifest, host_version, RESERVED_NAMESPACES)?;

    let namespace = validated.manifest.namespace.0.clone();
    let source = request.source;
    let record = PluginRecord {
        manifest: validated.manifest,
        state: PluginLifecycleState::Installed,
        source,
        manifest_checksum_sha256,
    };

    update_registry(registry_path, |registry| {
        if registry.plugins.contains_key(&namespace) {
            return Err(PluginError::NamespaceConflict(namespace.clone()));
        }
        ensure_aliases_do_not_conflict(registry, &record)?;
        registry.plugins.insert(namespace.clone(), record.clone());
        Ok(())
    })?;

    Ok(record)
}

fn ensure_aliases_do_not_conflict(
    registry: &PluginRegistry,
    candidate: &PluginRecord,
) -> Result<(), PluginError> {
    let mut existing_aliases = BTreeSet::new();
    for plugin in registry.plugins.values() {
        for alias in &plugin.manifest.aliases {
            existing_aliases.insert(alias.to_ascii_lowercase());
        }
    }

    for alias in &candidate.manifest.aliases {
        if existing_aliases.contains(&alias.to_ascii_lowercase()) {
            return Err(PluginError::AliasConflict(alias.clone()));
        }
    }

    Ok(())
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
        let plugin = registry
            .plugins
            .get_mut(namespace)
            .ok_or_else(|| PluginError::PluginNotFound(namespace.to_string()))?;
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

/// Return deterministic plugin load order contract.
pub fn plugin_load_order(registry_path: &Path) -> Result<Vec<PluginLoadEntry>, PluginError> {
    let registry = load_registry(registry_path)?;
    let mut items: Vec<PluginLoadEntry> = registry
        .plugins
        .iter()
        .map(|(namespace, record)| PluginLoadEntry {
            namespace: namespace.clone(),
            state: record.state,
        })
        .collect();

    items.sort_by(|left, right| {
        let left_rank = state_rank(left.state);
        let right_rank = state_rank(right.state);
        left_rank.cmp(&right_rank).then_with(|| left.namespace.cmp(&right.namespace))
    });

    Ok(items)
}

fn state_rank(state: PluginLifecycleState) -> u8 {
    match state {
        PluginLifecycleState::Enabled => 0,
        PluginLifecycleState::Installed | PluginLifecycleState::Validated => 1,
        PluginLifecycleState::Disabled => 2,
        PluginLifecycleState::Discovered => 3,
        PluginLifecycleState::Incompatible => 4,
        PluginLifecycleState::Broken => 5,
    }
}

/// Scan plugin directory tree for manifests at `<plugin-dir>/*/plugin.json`.
pub fn discover_plugin_manifests(plugins_dir: &Path) -> Result<Vec<PathBuf>, PluginError> {
    if !plugins_dir.exists() {
        return Ok(Vec::new());
    }

    let mut manifests = Vec::new();
    for entry in fs::read_dir(plugins_dir)? {
        let entry = entry?;
        let plugin_dir = entry.path();
        if !plugin_dir.is_dir() {
            continue;
        }

        let manifest_path = plugin_dir.join("plugin.json");
        if manifest_path.exists() {
            manifests.push(manifest_path);
        }
    }
    manifests.sort();
    Ok(manifests)
}

/// Refresh discovery cache from plugin directory scan.
pub fn refresh_discovery_cache(
    cache: &mut PluginDiscoveryCache,
    plugins_dir: &Path,
) -> Result<(), PluginError> {
    let discovered = discover_plugin_manifests(plugins_dir)?;
    let mut manifests = BTreeMap::new();

    for manifest_path in discovered {
        let text = fs::read_to_string(&manifest_path)?;
        let manifest = parse_manifest_v1(&text)?;
        manifests.insert(manifest.namespace.0, manifest_path);
    }

    cache.root = plugins_dir.to_path_buf();
    cache.manifests = manifests;
    cache.last_updated_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    Ok(())
}

/// Generate load-time diagnostics for broken or incompatible plugins.
pub fn load_time_diagnostics(
    registry_path: &Path,
    host_version: &str,
) -> Result<Vec<PluginLoadDiagnostic>, PluginError> {
    let registry = load_registry(registry_path)?;
    let mut diagnostics = Vec::new();

    for (namespace, record) in &registry.plugins {
        if record.state == PluginLifecycleState::Broken {
            diagnostics.push(PluginLoadDiagnostic {
                namespace: namespace.clone(),
                severity: "error".to_string(),
                message: "plugin is marked broken".to_string(),
            });
            continue;
        }

        if !is_version_compatible(&record.manifest.compatibility, host_version)? {
            diagnostics.push(PluginLoadDiagnostic {
                namespace: namespace.clone(),
                severity: "warning".to_string(),
                message: format!("plugin compatibility does not include host {host_version}"),
            });
        }

        if record.manifest.kind == PluginKind::ExternalExec
            && !Path::new(&record.manifest.entrypoint).exists()
        {
            diagnostics.push(PluginLoadDiagnostic {
                namespace: namespace.clone(),
                severity: "error".to_string(),
                message: "external-exec entrypoint was not found".to_string(),
            });
        }
    }

    Ok(diagnostics)
}

/// Try to repair a corrupted registry by quarantining the old file and writing a fresh empty registry.
pub fn self_repair_registry(path: &Path) -> Result<PluginRegistry, PluginError> {
    match load_registry(path) {
        Ok(registry) => Ok(registry),
        Err(PluginError::RegistryCorrupted) => {
            if path.exists() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |duration| duration.as_secs());
                let quarantine = path.with_extension(format!("corrupt-{timestamp}.json"));
                fs::rename(path, quarantine)?;
            }
            let repaired = PluginRegistry::default();
            save_registry(path, &repaired)?;
            Ok(repaired)
        }
        Err(error) => Err(error),
    }
}

/// Execute delegated plugin contract after capability guard checks.
pub fn execute_delegated_plugin(
    manifest: &PluginManifestV1,
    required_capability: &str,
) -> Result<String, PluginError> {
    if manifest.kind != PluginKind::Delegated && manifest.kind != PluginKind::Python {
        return Err(PluginError::UnsupportedKind(manifest.kind));
    }

    if !manifest
        .capabilities
        .iter()
        .any(|capability: &PluginCapability| capability.name == required_capability)
    {
        return Err(PluginError::MissingCapability(required_capability.to_string()));
    }

    Ok(format!(
        "delegated:{}:{}",
        manifest.namespace.0, manifest.entrypoint
    ))
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
    fn validates_and_rejects_reserved_core_and_future_namespaces() {
        let manifest = parse_manifest_v1(&sample_manifest("community")).expect("parse");
        let validated = validate_manifest(manifest, "0.1.0", RESERVED_NAMESPACES).expect("valid");
        assert_eq!(validated.state, PluginLifecycleState::Validated);

        let reserved_manifest = parse_manifest_v1(&sample_manifest("cli")).expect("parse");
        let error = validate_manifest(reserved_manifest, "0.1.0", RESERVED_NAMESPACES)
            .expect_err("reserved namespace must fail");
        assert!(matches!(error, PluginError::ReservedNamespace(_)));

        let future_manifest = parse_manifest_v1(&sample_manifest("atlas")).expect("parse");
        let future_error = validate_manifest(future_manifest, "0.1.0", RESERVED_NAMESPACES)
            .expect_err("future namespace must fail");
        assert!(matches!(future_error, PluginError::FutureNamespaceConflict(_)));
    }
}
