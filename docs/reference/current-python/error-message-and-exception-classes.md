# Error Message and Exception Classes

## Source of truth
- `src/bijux_cli/core/errors.py`
- `src/bijux_cli/services/errors.py`
- `src/bijux_cli/plugins/metadata.py`
- `src/bijux_cli/infra/serializer.py`
- `src/bijux_cli/core/exit_policy.py`

## Exception classes
- `BijuxError`
- `UserInputError`
- `ConfigError`
- `PluginError`
- `InternalError`
- `ServiceError`
- `PluginMetadataError`
- `SerializationError`
- `ExitIntentError`

## Error categories used for exit mapping
- `usage`
- `ascii`
- `user_input`
- `config`
- `plugin`
- `internal`
- `aborted`

## Message shape behavior
- Structured error payloads typically include `error`, `code`, `failure`, and `command`.
- Optional debug context includes traceback when log policy enables it.

## Structured failure classes observed in source
- `args`
- `ascii`
- `ascii_env`
- `ascii_error`
- `clear_failed`
- `config`
- `config_unreadable`
- `control_char_error`
- `cookiecutter_missing`
- `create_dir_failed`
- `delete_failed`
- `dir_not_empty`
- `emit`
- `empty_key`
- `entrypoint_missing`
- `export_failed`
- `file_locked`
- `format`
- `get_failed`
- `group_by`
- `health_error`
- `health_unavailable`
- `import_error`
- `import_failed`
- `internal`
- `interval`
- `invalid_argument`
- `invalid_color`
- `invalid_format`
- `invalid_key`
- `invalid_log_level`
- `invalid_name`
- `io_fail`
- `limit`
- `list_failed`
- `load_failed`
- `metadata_corrupt`
- `metadata_error`
- `missing_argument`
- `name_conflict`
- `negative`
- `no_template`
- `not_dir`
- `not_found`
- `not_installed`
- `null_byte`
- `output_dir`
- `output_file`
- `output_write`
- `permission_denied`
- `pip_install_failed`
- `pip_uninstall_failed`
- `plugin_json_invalid`
- `plugin_json_missing`
- `reload_failed`
- `remove_failed`
- `reserved_keyword`
- `scaffold_failed`
- `serialize`
- `service_unavailable`
- `set_failed`
- `sort`
- `symlink_dir`
- `symlink_path`
- `timeout`
- `unexpected`
- `unset_failed`
- `watch_fmt`
- `write`
