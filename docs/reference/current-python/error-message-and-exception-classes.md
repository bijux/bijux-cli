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
