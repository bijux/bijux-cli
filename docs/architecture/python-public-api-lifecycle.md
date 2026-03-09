# Python Public API Lifecycle

This document records which Python-facing APIs remain stable and which are candidates for deprecation after baseline.

## Stable APIs to keep

- `version()`
- `execution_facade(argv)`
- `execution_facade_with_status(argv)`
- `command_tree_introspection()`
- `run_cli(argv)`
- `get_version()`
- `get_command_tree()`
- `post_install_diagnostics()`
- `migration_warnings()`
- `ensure_native_extension()`
- Exceptions: `BijuxPythonError`, `NativeExtensionUnavailable`, `PlatformWheelUnavailable`

These are retained because they are direct runtime integration points for current automation and compatibility workflows.

## Compatibility APIs retained with deprecation intent

- `deprecated_version_api()`

This API remains available for compatibility but is explicitly marked deprecated and should not be used in new integrations.

## Helper APIs kept as provisional

- `config_resolution_helpers(home_dir)`
- `plugin_registry_inspection(registry_file)`
- `install_path_helpers(home_dir)`
- `path_ambiguity_detection_message(...)`
- `side_by_side_install_report(...)`
- `simulate_pip_uninstall_cleanup(...)`
- `simulate_pip_upgrade_preserves_state(...)`
- `check_embedded_binary_compatibility(expected_version)`
- `check_python_runtime_supported(...)`

These are currently used for diagnostics, packaging checks, and migration flows. They remain available but may be tightened once equivalent command-level diagnostics are fully standardized.
