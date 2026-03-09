# Python Bridge API Review

Date: 2026-03-09

## Stability review

Stable bridge entrypoints:

- `version_binding_api`
- `doctor_binding_api`
- `status_binding_api`
- `cli_status_binding_api`
- `plugins_list_binding_api`
- `repl_bootstrap_binding_api`
- `execution_facade_api`
- `execution_outcome_api`
- `schema_export_helpers_api`
- `config_resolution_api`
- `install_path_helpers_api`
- `plugin_registry_inspection_api`

Stable compatibility exports:

- `discover_compatibility_paths`
- `default_compatibility_paths`
- `load_compatibility_config`
- `write_compatibility_config`
- `acquire_state_lock`
- `ensure_history_file`
- `ensure_plugins_dir`

## Minimal-surface review

The bridge public surface is constrained to:

1. Command execution adapters.
2. Compatibility path/config helpers.
3. Error-kind conversion helpers for Python exception mapping.

The bridge explicitly does not own:

1. Command parsing law.
2. Route resolution law.
3. Core execution semantics.
4. Output envelope rendering policy.

Those remain in `bijux-cli-core`, `bijux-cli-routing`, and `bijux-cli-output`.
