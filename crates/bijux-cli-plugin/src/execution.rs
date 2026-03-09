#![forbid(unsafe_code)]

use bijux_cli_contracts::{PluginCapability, PluginKind, PluginManifestV1};

use crate::errors::PluginError;

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

    Ok(format!("delegated:{}:{}", manifest.namespace.0, manifest.entrypoint))
}
