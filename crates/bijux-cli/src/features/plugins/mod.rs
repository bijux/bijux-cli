#![forbid(unsafe_code)]
//! Plugin registration and lifecycle boundaries.

mod constants;
mod diagnostics;
mod discovery;
mod entrypoint;
mod errors;
mod manifest;
mod models;
pub(crate) mod operations;
mod registry;
/// Plugin scaffolding helpers for manifest and starter files.
pub(crate) mod scaffold;

use std::path::Path;

#[allow(unused_imports)]
pub use constants::{
    is_reserved_namespace, CORE_NAMESPACES, KNOWN_BIJUX_PROJECT_NAMESPACES, RESERVED_NAMESPACES,
};
#[allow(unused_imports)]
pub use diagnostics::{
    compatibility_warnings, load_time_diagnostics, prune_registry_backup, self_repair_registry,
};
#[allow(unused_imports)]
pub use discovery::{
    discover_plugin_manifests, refresh_discovery_cache, registry_path_from_plugins_dir,
};
pub use entrypoint::{
    delegated_entrypoint_candidates, installed_manifest_root, resolve_delegated_entrypoint,
};
pub use errors::PluginError;
pub use manifest::{parse_manifest_v1, validate_manifest};
#[allow(unused_imports)]
pub use models::{
    InstallPluginRequest, PluginDiscoveryCache, PluginDoctorReport, PluginLoadDiagnostic,
    PluginLoadEntry, PluginOriginMetadata, PluginRecord, PluginRegistry, PluginTrustLevel,
    ValidatedPlugin,
};
#[allow(unused_imports)]
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
