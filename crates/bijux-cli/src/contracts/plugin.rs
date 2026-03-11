use schemars::JsonSchema;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use super::command::Namespace;

/// Stable compatibility range contract for plugins and features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompatibilityRange {
    /// Minimum supported version inclusive.
    pub min_inclusive: String,
    /// Optional maximum supported version exclusive.
    pub max_exclusive: Option<String>,
}

impl CompatibilityRange {
    /// Build a validated compatibility range.
    pub fn new(min_inclusive: &str, max_exclusive: Option<&str>) -> Result<Self, String> {
        let _ = Version::parse(min_inclusive)
            .map_err(|error| format!("invalid min_inclusive semver: {error}"))?;
        if let Some(max) = max_exclusive {
            let _ = Version::parse(max)
                .map_err(|error| format!("invalid max_exclusive semver: {error}"))?;
        }
        Ok(Self {
            min_inclusive: min_inclusive.to_string(),
            max_exclusive: max_exclusive.map(ToString::to_string),
        })
    }

    /// Check whether a host version is supported by this range.
    pub fn supports_host(&self, host_version: &str) -> Result<bool, String> {
        let host = Version::parse(host_version)
            .map_err(|error| format!("invalid host semver: {error}"))?;
        let min_req = VersionReq::parse(&format!(">={}", self.min_inclusive))
            .map_err(|error| format!("invalid min constraint: {error}"))?;
        if !min_req.matches(&host) {
            return Ok(false);
        }
        if let Some(max) = &self.max_exclusive {
            let max_req = VersionReq::parse(&format!("<{max}"))
                .map_err(|error| format!("invalid max constraint: {error}"))?;
            return Ok(max_req.matches(&host));
        }
        Ok(true)
    }
}

/// Stable plugin capability declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginCapability {
    /// Capability identifier.
    pub name: String,
    /// Optional capability version.
    pub version: Option<String>,
}

impl PluginCapability {
    /// Build a validated plugin capability declaration.
    pub fn new(name: &str, version: Option<&str>) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("capability name cannot be empty".to_string());
        }
        Ok(Self {
            name: name.to_string(),
            version: version.map(ToString::to_string),
        })
    }
}

/// Stable plugin kind declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum PluginKind {
    /// Future in-process plugin ABI.
    Native,
    /// Delegated plugin loaded through host contract bridge.
    #[default]
    Delegated,
    /// Python delegated plugin runtime.
    Python,
    /// External executable plugin.
    ExternalExec,
}

/// Stable plugin lifecycle state in registry and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum PluginLifecycleState {
    /// Artifact located during discovery.
    Discovered,
    /// Manifest and contract validation passed.
    Validated,
    /// Plugin installed in registry.
    Installed,
    /// Plugin actively enabled for routing.
    Enabled,
    /// Plugin present but inactive.
    Disabled,
    /// Plugin failed validation or runtime loading.
    Broken,
    /// Plugin failed compatibility checks.
    Incompatible,
}

/// Stable plugin manifest contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginManifestV1 {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin schema version.
    pub schema_version: String,
    /// Manifest contract version.
    pub manifest_version: String,
    /// Compatibility range for host CLI.
    pub compatibility: CompatibilityRange,
    /// Declared top-level namespace.
    pub namespace: Namespace,
    /// Plugin execution kind.
    #[serde(default)]
    pub kind: PluginKind,
    /// Declared command aliases.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Plugin entrypoint (binary path or module symbol).
    pub entrypoint: String,
    /// Declared capabilities.
    pub capabilities: Vec<PluginCapability>,
}

impl PluginManifestV1 {
    /// Build a validated plugin manifest.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        version: &str,
        schema_version: &str,
        manifest_version: &str,
        compatibility: CompatibilityRange,
        namespace: Namespace,
        kind: PluginKind,
        aliases: Vec<String>,
        entrypoint: &str,
        capabilities: Vec<PluginCapability>,
    ) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("plugin name cannot be empty".to_string());
        }
        if version.trim().is_empty() {
            return Err("plugin version cannot be empty".to_string());
        }
        if schema_version.trim().is_empty() {
            return Err("plugin schema_version cannot be empty".to_string());
        }
        if manifest_version.trim().is_empty() {
            return Err("plugin manifest_version cannot be empty".to_string());
        }
        if entrypoint.trim().is_empty() {
            return Err("plugin entrypoint cannot be empty".to_string());
        }
        Ok(Self {
            name: name.to_string(),
            version: version.to_string(),
            schema_version: schema_version.to_string(),
            manifest_version: manifest_version.to_string(),
            compatibility,
            namespace,
            kind,
            aliases,
            entrypoint: entrypoint.to_string(),
            capabilities,
        })
    }
}
