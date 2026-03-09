# Install API Stability

## Stable public surface

Metadata and strategy:

- `installer_marker`
- `canonical_crate_name`
- `publish_compatibility_package_alias`
- `cargo_install_strategy`
- `pip_install_strategy`
- `has_secondary_executable_conflict`

Path and diagnostics:

- `discover_path_binaries`
- `resolve_active_binary`
- `detect_stale_wrapper_scripts`
- `legacy_installer_conflicts`
- `install_health_report`
- `initialize_first_run_state`

Compatibility config and state:

- `default_compatibility_paths`
- `discover_compatibility_paths`
- `parse_compatibility_config`
- `load_compatibility_config`
- `write_compatibility_config`
- `acquire_state_lock`
- `ensure_history_file`
- `ensure_plugins_dir`
- `run_config_migrations`

Completion and user guidance:

- `completion_script`
- `completion_file_path`
- `detect_shell`
- `post_install_hint`
- `pip_compatibility_note`
- `cargo_compatibility_note`

## Internal-only behavior

The following are internal and may change without compatibility guarantees:

- private path normalization helpers
- private install diagnostics helper composition details
- lock file contents and marker file payload bytes
