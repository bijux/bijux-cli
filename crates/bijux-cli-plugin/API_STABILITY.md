# Plugin API Stability

## Stable public surface
The following APIs are treated as stable in this baseline:
- `install_plugin`
- `uninstall_plugin`
- `enable_plugin`
- `disable_plugin`
- `inspect_plugin`
- `list_plugins`
- `plugin_doctor`
- `plugin_load_order`
- `discover_plugin_manifests`
- `refresh_discovery_cache`
- `load_time_diagnostics`
- `compatibility_warnings`
- `self_repair_registry`
- `execute_delegated_plugin`
- `registry_path_from_plugins_dir`

## Architectural boundaries
- `manifest.rs`: parsing and validation only.
- `registry.rs`: persistence and mutation lifecycle only.
- `discovery.rs`: filesystem discovery and cache only.
- `diagnostics.rs`: read-only diagnostics and repair utilities.
- `execution.rs`: capability checks and delegated execution wiring.

## Compatibility contract
- Error categories in `PluginError` are machine-significant.
- Registry file path remains `<plugins-dir>/registry.json`.
- Deterministic ordering is required for list/load-order outputs.

## Change policy
- Additive API growth is preferred.
- Breaking public signature changes require explicit compatibility note and migration guidance.
