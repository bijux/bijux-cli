#![forbid(unsafe_code)]
//! Plugin registry and diagnostics facade.

pub use crate::features::plugins::{
    compatibility_check, compatibility_warnings, disable_plugin, discover_plugin_manifests,
    enable_plugin, inspect_plugin, install_plugin, install_plugin_with_reserved, list_plugins,
    load_registry, load_time_diagnostics, parse_manifest_v1, plugin_doctor, plugin_load_order,
    plugin_origin_metadata, prune_registry_backup, refresh_discovery_cache,
    registry_path_from_plugins_dir, save_registry, self_repair_registry, uninstall_plugin,
    update_registry, validate_manifest, InstallPluginRequest, PluginDiscoveryCache,
    PluginDoctorReport, PluginError, PluginLoadDiagnostic, PluginLoadEntry, PluginOriginMetadata,
    PluginRecord, PluginRegistry, PluginTrustLevel, ValidatedPlugin, CORE_NAMESPACES,
    KNOWN_BIJUX_PROJECT_NAMESPACES, RESERVED_NAMESPACES,
};
