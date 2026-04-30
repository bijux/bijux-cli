#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::contracts::{CompatibilityRange, Namespace, PluginKind, PluginManifestV2};
use semver::Version;

use super::constants::{is_reserved_namespace, CORE_NAMESPACES};
use super::errors::PluginError;
use super::models::ValidatedPlugin;
use crate::contracts::known_bijux_tool_namespaces;

/// Parse `PluginManifestV2` from JSON text.
pub fn parse_manifest_v2(text: &str) -> Result<PluginManifestV2, PluginError> {
    serde_json::from_str(text).map_err(|error| PluginError::ManifestParse(error.to_string()))
}

/// Validate plugin manifest against host compatibility and namespace rules.
pub fn validate_manifest(
    manifest: PluginManifestV2,
    host_version: &str,
    reserved_namespaces: &[&str],
) -> Result<ValidatedPlugin, PluginError> {
    validate_required_fields(&manifest)?;
    validate_namespace_format(&manifest.namespace)?;
    reject_reserved_namespace(&manifest.namespace, reserved_namespaces)?;
    reject_core_namespace(&manifest.namespace)?;
    reject_known_bijux_project_namespace(&manifest.namespace)?;
    validate_aliases(&manifest.namespace, &manifest.aliases)?;
    validate_compatibility(&manifest.compatibility, host_version)?;
    validate_entrypoint_and_kind(&manifest)?;

    Ok(ValidatedPlugin { manifest, state: crate::contracts::PluginLifecycleState::Validated })
}

fn validate_required_fields(manifest: &PluginManifestV2) -> Result<(), PluginError> {
    if manifest.name.trim().is_empty() {
        return Err(PluginError::InvalidField("name".to_string()));
    }
    if manifest.version.trim().is_empty() {
        return Err(PluginError::InvalidField("version".to_string()));
    }
    Version::parse(&manifest.version)
        .map_err(|_| PluginError::InvalidField("version".to_string()))?;
    if manifest.schema_version.trim().is_empty() {
        return Err(PluginError::InvalidField("schema_version".to_string()));
    }
    if manifest.schema_version != "v2" {
        return Err(PluginError::InvalidField("schema_version".to_string()));
    }
    if manifest.manifest_version.trim().is_empty() {
        return Err(PluginError::InvalidField("manifest_version".to_string()));
    }
    if manifest.manifest_version != "v2" {
        return Err(PluginError::InvalidField("manifest_version".to_string()));
    }
    Ok(())
}

pub(crate) fn validate_namespace_text(namespace: &str) -> Result<(), PluginError> {
    let bytes = namespace.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return Err(PluginError::InvalidNamespace(namespace.to_string()));
    }

    if !bytes.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
    {
        return Err(PluginError::InvalidNamespace(namespace.to_string()));
    }

    if namespace.contains("--") || namespace.ends_with('-') {
        return Err(PluginError::InvalidNamespace(namespace.to_string()));
    }

    Ok(())
}

fn validate_namespace_format(namespace: &Namespace) -> Result<(), PluginError> {
    validate_namespace_text(namespace.0.as_str())
}

fn reject_reserved_namespace(namespace: &Namespace, reserved: &[&str]) -> Result<(), PluginError> {
    if is_reserved_namespace(&namespace.0, reserved) {
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

fn reject_known_bijux_project_namespace(namespace: &Namespace) -> Result<(), PluginError> {
    if known_bijux_tool_namespaces().iter().any(|value| *value == namespace.0) {
        return Err(PluginError::FutureNamespaceConflict(namespace.0.clone()));
    }
    Ok(())
}

fn validate_aliases(namespace: &Namespace, aliases: &[String]) -> Result<(), PluginError> {
    let mut seen = BTreeSet::new();
    for alias in aliases {
        validate_alias_format(alias)?;
        if alias == &namespace.0 {
            return Err(PluginError::AliasNamespaceConflict(alias.clone()));
        }
        if is_reserved_namespace(alias, &[])
            || CORE_NAMESPACES.iter().any(|value| *value == alias)
            || known_bijux_tool_namespaces().iter().any(|value| *value == alias)
        {
            return Err(PluginError::ReservedAlias(alias.clone()));
        }
        if !seen.insert(alias.to_ascii_lowercase()) {
            return Err(PluginError::DuplicateAlias(alias.clone()));
        }
    }
    Ok(())
}

fn validate_alias_format(alias: &str) -> Result<(), PluginError> {
    if alias.trim().is_empty() || alias.trim() != alias {
        return Err(PluginError::InvalidAlias(alias.to_string()));
    }

    let bytes = alias.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return Err(PluginError::InvalidAlias(alias.to_string()));
    }

    if !bytes.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
    {
        return Err(PluginError::InvalidAlias(alias.to_string()));
    }

    if alias.contains("--") || alias.ends_with('-') {
        return Err(PluginError::InvalidAlias(alias.to_string()));
    }

    Ok(())
}

fn validate_compatibility(
    range: &CompatibilityRange,
    host_version: &str,
) -> Result<(), PluginError> {
    let min = Version::parse(&range.min_inclusive)
        .map_err(|_| PluginError::InvalidField("compatibility.min_inclusive".to_string()))?;
    if let Some(max_exclusive) = &range.max_exclusive {
        let max = Version::parse(max_exclusive)
            .map_err(|_| PluginError::InvalidField("compatibility.max_exclusive".to_string()))?;
        if max <= min {
            return Err(PluginError::InvalidField("compatibility.max_exclusive".to_string()));
        }
    }

    if !is_version_compatible(range, host_version)? {
        return Err(PluginError::IncompatibleVersion { host_version: host_version.to_string() });
    }
    Ok(())
}

pub(crate) fn is_version_compatible(
    range: &CompatibilityRange,
    host_version: &str,
) -> Result<bool, PluginError> {
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

fn validate_entrypoint_and_kind(manifest: &PluginManifestV2) -> Result<(), PluginError> {
    if manifest.entrypoint.trim().is_empty() {
        return Err(PluginError::InvalidEntrypoint { kind: manifest.kind });
    }

    match manifest.kind {
        PluginKind::Delegated | PluginKind::Python => {
            let Some((module_name, callable_name)) = manifest.entrypoint.split_once(':') else {
                return Err(PluginError::InvalidEntrypoint { kind: manifest.kind });
            };
            validate_symbol_path(module_name, manifest.kind)?;
            validate_symbol_path(callable_name, manifest.kind)?;
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

fn validate_symbol_path(path: &str, kind: PluginKind) -> Result<(), PluginError> {
    let segments: Vec<&str> = path.split('.').map(str::trim).collect();
    if segments.is_empty() || segments.iter().any(|segment| segment.is_empty()) {
        return Err(PluginError::InvalidEntrypoint { kind });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_manifest;
    use crate::api::version::runtime_semver;
    use crate::contracts::{
        CompatibilityRange, Namespace, PluginCapability, PluginKind, PluginManifestV2,
        PluginTrustClass,
    };
    use semver::{Prerelease, Version};

    fn current_plugin_host_floor() -> String {
        let runtime = Version::parse(runtime_semver()).expect("runtime semver");
        let mut floor = Version::new(runtime.major, runtime.minor, runtime.patch);
        if !runtime.pre.is_empty() {
            let channel =
                runtime.pre.as_str().split('.').next().expect("runtime prerelease channel");
            floor.pre = Prerelease::new(channel).expect("prerelease channel");
        }
        floor.to_string()
    }

    fn current_plugin_host_ceiling() -> String {
        let runtime = Version::parse(runtime_semver()).expect("runtime semver");
        Version::new(runtime.major + 1, 0, 0).to_string()
    }

    fn sample_manifest() -> PluginManifestV2 {
        PluginManifestV2 {
            name: "sample".to_string(),
            version: "0.1.0".to_string(),
            schema_version: "v2".to_string(),
            manifest_version: "v2".to_string(),
            compatibility: CompatibilityRange {
                min_inclusive: current_plugin_host_floor(),
                max_exclusive: Some(current_plugin_host_ceiling()),
            },
            namespace: Namespace::new("sample").expect("namespace"),
            kind: PluginKind::Python,
            trust_class: PluginTrustClass::Community,
            aliases: Vec::new(),
            entrypoint: "plugin:main".to_string(),
            capabilities: vec![PluginCapability {
                name: "inspect".to_string(),
                version: Some("1".to_string()),
            }],
        }
    }

    #[test]
    fn validate_manifest_rejects_non_v2_schema_versions() {
        let mut manifest = sample_manifest();
        manifest.schema_version = "1".to_string();
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("schema version");
        assert_eq!(error.to_string(), "plugin manifest field invalid: schema_version");
    }

    #[test]
    fn validate_manifest_rejects_non_v2_manifest_versions() {
        let mut manifest = sample_manifest();
        manifest.manifest_version = "1".to_string();
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("manifest version");
        assert_eq!(error.to_string(), "plugin manifest field invalid: manifest_version");
    }

    #[test]
    fn validate_manifest_rejects_non_semver_plugin_versions() {
        let mut manifest = sample_manifest();
        manifest.version = "release".to_string();
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("plugin version");
        assert_eq!(error.to_string(), "plugin manifest field invalid: version");
    }

    #[test]
    fn validate_manifest_rejects_invalid_compatibility_windows() {
        let mut manifest = sample_manifest();
        manifest.compatibility.max_exclusive = Some(current_plugin_host_floor());
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("compatibility");
        assert_eq!(error.to_string(), "plugin manifest field invalid: compatibility.max_exclusive");
    }

    #[test]
    fn validate_manifest_rejects_alias_matching_namespace() {
        let mut manifest = sample_manifest();
        manifest.aliases = vec!["sample".to_string()];
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("alias conflict");
        assert_eq!(error.to_string(), "plugin alias conflicts with plugin namespace: sample");
    }

    #[test]
    fn validate_manifest_rejects_invalid_alias_format() {
        let mut manifest = sample_manifest();
        manifest.aliases = vec!["bad alias".to_string()];
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("alias format");
        assert_eq!(error.to_string(), "plugin alias is invalid: bad alias");
    }

    #[test]
    fn validate_manifest_rejects_reserved_aliases() {
        let mut manifest = sample_manifest();
        manifest.aliases = vec!["cli".to_string()];
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("reserved alias");
        assert_eq!(error.to_string(), "plugin alias is reserved: cli");
    }

    #[test]
    fn validate_manifest_rejects_python_entrypoints_without_a_callable_separator() {
        let mut manifest = sample_manifest();
        manifest.entrypoint = "plugin.main".to_string();
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("entrypoint separator");
        assert_eq!(error.to_string(), "plugin entrypoint is invalid for kind Python");
    }

    #[test]
    fn validate_manifest_rejects_python_entrypoints_with_empty_segments() {
        let mut manifest = sample_manifest();
        manifest.entrypoint = "plugin.:main".to_string();
        let host = current_plugin_host_floor();
        let error = validate_manifest(manifest, &host, &[]).expect_err("empty module segment");
        assert_eq!(error.to_string(), "plugin entrypoint is invalid for kind Python");
    }
}
