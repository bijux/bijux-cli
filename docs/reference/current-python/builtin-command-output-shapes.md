# Built-in Command Output Shapes

## Source of truth
- Command docstrings under `src/bijux_cli/cli/commands/`
- Plugin command docstrings under `src/bijux_cli/cli/plugins/commands/`

## Shared shape conventions
- Success payloads are structured dictionaries (JSON/YAML) with command-specific keys.
- Error payloads commonly include `error` and `code`, with optional `failure`, `command`, and runtime metadata.

## Representative built-in output contracts
- `status`: status and runtime probe payload with optional diagnostics.
- `version`: version metadata payload.
- `doctor`: health diagnostics payload with healthy/unhealthy outcomes.
- `audit`: `{"status": "completed"}` or `{"status": "dry-run"}` style payload.
- `config get`: key/value lookup payload.
- `config list`: map of config keys and values.
- `history`: entries list payload.
- `memory get|set|list|delete|clear`: key/value store payloads and status markers.
- `plugins list`: `{"plugins": [{"name": str, "version": str, "enabled": bool}, ...]}`.
- `plugins info`: plugin metadata payload.
- `plugins install`: installed or dry-run payload with package/plugins fields.
- `plugins uninstall`: `{"status": "uninstalled", "plugin": str}`.
- `plugins check`: health status payload (`healthy` or `unhealthy`).

## Notes
- Output contracts are command-local and not yet centrally schema-locked.
