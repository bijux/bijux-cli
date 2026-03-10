#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use bijux_cli_routing::{CompatibilityRange, Namespace, PluginKind, PluginManifestV1};
use semver::Version;

use super::constants::{is_reserved_namespace, CORE_NAMESPACES, FUTURE_PRODUCT_NAMESPACES};
use super::errors::PluginError;
use super::models::ValidatedPlugin;

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

    Ok(ValidatedPlugin { manifest, state: bijux_cli_routing::PluginLifecycleState::Validated })
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

    if !bytes.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
    {
        return Err(PluginError::InvalidNamespace(raw.to_string()));
    }

    if raw.contains("--") || raw.ends_with('-') {
        return Err(PluginError::InvalidNamespace(raw.to_string()));
    }

    Ok(())
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

fn validate_compatibility(
    range: &CompatibilityRange,
    host_version: &str,
) -> Result<(), PluginError> {
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
        _ => return Err(PluginError::UnsupportedKind(manifest.kind)),
    }

    Ok(())
}
