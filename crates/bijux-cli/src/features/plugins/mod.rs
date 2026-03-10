#![forbid(unsafe_code)]
//! Plugin registration and lifecycle boundaries.

/// CLI command handlers for plugin lifecycle flows.
pub(crate) mod command;
mod constants;
mod diagnostics;
mod discovery;
mod errors;
mod manifest;
mod models;
mod registry;

use std::path::Path;

pub use constants::{
    is_reserved_namespace, CORE_NAMESPACES, FUTURE_PRODUCT_NAMESPACES, RESERVED_NAMESPACES,
};
pub use diagnostics::{
    compatibility_warnings, load_time_diagnostics, prune_registry_backup, self_repair_registry,
};
pub use discovery::{
    discover_plugin_manifests, refresh_discovery_cache, registry_path_from_plugins_dir,
};
pub use errors::PluginError;
pub use manifest::{parse_manifest_v1, validate_manifest};
pub use models::{
    InstallPluginRequest, PluginDiscoveryCache, PluginDoctorReport, PluginLoadDiagnostic,
    PluginLoadEntry, PluginOriginMetadata, PluginRecord, PluginRegistry, PluginTrustLevel,
    ValidatedPlugin,
};
pub use registry::{
    compatibility_check, disable_plugin, enable_plugin, inspect_plugin,
    install_plugin as install_plugin_with_reserved, list_plugins, load_registry, plugin_doctor,
    plugin_load_order, plugin_origin_metadata, save_registry, uninstall_plugin, update_registry,
};

/// Install plugin into registry from manifest text.
pub fn install_plugin(
    registry_path: &Path,
    request: InstallPluginRequest,
    host_version: &str,
) -> Result<PluginRecord, PluginError> {
    registry::install_plugin(registry_path, request, host_version, RESERVED_NAMESPACES)
}
