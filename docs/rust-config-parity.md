# Rust Config Parity Baseline

This document locks the Python `config` command behavior as the Rust parity baseline.

## Scope

This baseline covers tasks 1-20 for the Rust config migration track.

## Python Config Subcommands Present

Captured from `bijux config --help`:

- `config` (root)
- `config list`
- `config get`
- `config set`
- `config unset`
- `config clear`
- `config export`
- `config load`
- `config reload`

## Frozen Baseline Behavior

### `config` root

- Exit code: `0`
- Behavior: returns active configuration listing as machine output.
- Golden: `artifacts/python-behavior/golden/config/config_root.json`

### `config get`

- Success path exit code: `0`
- Missing key exit code: `2`
- Missing key emits structured error on stdout.
- Goldens:
  - `artifacts/python-behavior/golden/config/config_get_sample.json`
  - `artifacts/python-behavior/golden/config/config_get_missing.json`

### `config set`

- Exit code: `0` for valid `KEY=VALUE` input.
- Behavior: returns updated key/value payload.
- Golden: `artifacts/python-behavior/golden/config/config_set_sample.json`

### `config unset`

- Exit code: `0` for existing key removal in baseline capture.
- Golden: `artifacts/python-behavior/golden/config/config_unset_sample.json`

### `config clear`

- Exit code: `0`
- Behavior: clears active config store.
- Golden: `artifacts/python-behavior/golden/config/config_clear.json`

### `config export`

- Baseline capture without required path returns:
  - Exit code: `2`
  - Structured error payload (`Missing parameter: path`) on stdout.
- Goldens:
  - `artifacts/python-behavior/golden/config/config_export_env.json`
  - `artifacts/python-behavior/golden/config/config_export_json.json`
  - `artifacts/python-behavior/golden/config/config_export_yaml.json`

### `config load`

- Exit code: `0` when loading valid file.
- Golden: `artifacts/python-behavior/golden/config/config_load.json`

### `config reload`

- Exit code: `0` in captured baseline context.
- Golden: `artifacts/python-behavior/golden/config/config_reload.json`

## Captured Help Goldens

- `config_root_help.json`
- `config_list_help.json`
- `config_get_help.json`
- `config_set_help.json`
- `config_unset_help.json`
- `config_clear_help.json`
- `config_export_help.json`
- `config_load_help.json`
- `config_reload_help.json`

All stored in `artifacts/python-behavior/golden/config/`.

## Output, Exit, and Stream Baseline Artifacts

- Capture manifest: `artifacts/python-behavior/golden/config/capture-summary.json`
- Each per-command golden stores:
  - `stdout`
  - `stderr`
  - `exit_code`
  - effective environment overrides for reproducibility

## File Layout Assumptions

Baseline references:

- `docs/reference/current-python/config-paths-and-environment-variables.md`
- `docs/architecture/config-python-semantics-audit.md`

Locked assumptions:

- Default config file path is `~/.bijux/.env`.
- Active config path can be overridden by `BIJUXCLI_CONFIG`.
- Parent directories are created on write paths.
- Missing config file on read is treated as empty config map.

## Key Validation Rules

Locked from Python semantics audit:

- Normalization trims and removes `BIJUXCLI_` prefix.
- Effective key comparisons are case-insensitive after normalization.
- Allowed baseline key pattern is alphanumeric plus underscore.

## Value Validation Rules

Locked from Python semantics audit:

- Values are string-based for config storage.
- Quoted/escaped pair parsing is supported in `config set` input.
- Control characters and non-ASCII are rejected in validated baseline paths.

## Path Resolution Rules

- Config path resolution: `BIJUXCLI_CONFIG` override, otherwise default `~/.bijux/.env`.
- Related path variables observed in baseline captures:
  - `BIJUXCLI_HISTORY_FILE`
  - `BIJUXCLI_PLUGINS_DIR`

## Must Decide Before Improvement

These ambiguities are explicitly locked as decision points before any Rust “better than Python” changes:

1. `config export` argument contract and format semantics when no path is supplied.
2. Exact parity target for `config` root vs `config list` output equivalence.
3. Stdin fallback behavior for `config set` when pair argument is omitted.
4. Whether missing-key and validation failures should remain stdout-routed or move to stderr in future policy.
5. Exact quoting and escape-decoding compatibility boundaries for edge-value inputs.

## Completion Matrix (Tasks 1-20)

- `1`: complete (`docs/rust-config-parity.md`)
- `2`: complete (subcommand list in this document)
- `3-10`: complete (frozen baseline sections per command)
- `11`: complete (help goldens in `artifacts/python-behavior/golden/config/`)
- `12`: complete (success output goldens)
- `13`: complete (error output goldens)
- `14`: complete (exit codes in per-command goldens and summary)
- `15`: complete (stdout/stderr captured per command)
- `16`: complete (file layout assumptions section)
- `17`: complete (key validation rules section)
- `18`: complete (value validation rules section)
- `19`: complete (path resolution rules section)
- `20`: complete (must-decide list)
