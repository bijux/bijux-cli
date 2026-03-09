# Built-in Command Output Shapes

## Source of truth
- Command docstrings under `src/bijux_cli/cli/commands/`
- Plugin command docstrings under `src/bijux_cli/cli/plugins/commands/`

## Shared shape conventions
- Success payloads are structured dictionaries (JSON/YAML) with command-specific keys.
- Error payloads commonly include `error` and `code`, with optional `failure`, `command`, and runtime metadata.

## Exhaustive built-in command-path inventory
- `atlas`: delegated external command output passthrough.
- `audit`: status payload (`completed` or `dry-run`) plus diagnostics fields.
- `config`: service status payload.
- `config list`: key/value map payload.
- `config get`: single key lookup payload.
- `config set`: mutation confirmation payload.
- `config unset`: mutation confirmation payload.
- `config export`: export destination/status payload.
- `config load`: load/import status payload.
- `config reload`: reload status payload.
- `config clear`: clear status payload.
- `dev`: developer status payload.
- `dev atlas`: delegated external command output passthrough.
- `dev di`: dependency graph payload.
- `dev list-products`: product binary discovery payload.
- `dev list-plugins`: plugin discovery payload.
- `docs`: diagnostics document generation payload.
- `doctor`: health diagnostics payload.
- `help`: help payload and command metadata payload.
- `history`: history service payload including `entries`.
- `history clear`: history clear status payload.
- `memory`: memory service payload.
- `memory list`: list payload.
- `memory get`: key lookup payload.
- `memory set`: write status payload.
- `memory delete`: delete status payload.
- `memory clear`: clear status payload.
- `memory resolve`: resolution payload.
- `plugins list`: plugin list payload.
- `plugins info`: plugin metadata payload.
- `plugins check`: plugin health payload.
- `plugins install`: install status payload (`installed` or `dry-run`).
- `plugins uninstall`: uninstall status payload.
- `plugins scaffold`: scaffold status payload.
- `repl`: REPL control/session status semantics.
- `sleep`: timing/status payload.
- `status`: runtime status and diagnostics payload.
- `version`: version metadata payload.

## Notes
- Output contracts are command-local and not yet centrally schema-locked.
