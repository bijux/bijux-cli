# Python Config Semantics Audit

This audit freezes Python behavior for configuration commands as implemented in `src/bijux_cli/cli/commands/config/*` and `src/bijux_cli/services/config/__init__.py`.

## Command Baseline

- `config` (root): lists all current key/value entries from config service.
- `config get KEY`:
  - Normalizes key: trims, removes `BIJUXCLI_` prefix, lowercases.
  - Reads env override first via `BIJUXCLI_<KEY_UPPER>`.
  - Falls back to loaded config map.
  - Missing key returns error payload and exit `2`.
- `config set KEY=VALUE`:
  - Accepts direct `KEY=VALUE` argument.
  - Reads from stdin only when argument omitted and stdin is not TTY.
  - Validates key pattern `^[A-Za-z0-9_]+$`.
  - Rejects empty key, non-ASCII key/value, and control characters in value.
  - Supports quoted values and escape decoding in the pair parser.
  - Persists with uppercase `BIJUXCLI_<KEY>` entries.
- `config unset KEY`: removes key if present.
- `config clear`: clears in-memory config and removes file.
- `config export`: supports env/json/yaml output forms.
- `config load`: imports external file and re-persists to active config path.
- `config reload`: reloads last-loaded path.

## File and Path Behavior

- Default path uses `BIJUXCLI_CONFIG` env or package default path.
- Missing default file on read is treated as empty config map.
- Missing explicit import path is an error.
- Writes are atomic via temp file + replace.
- Config writes create parent directories when needed.

## Validation Baseline

- Keys: case-insensitive after normalization; stored as lowercase in memory.
- Values: stored as strings; escaped in file output.
- Parsing errors on malformed lines (`=` missing) are surfaced as config errors.

## Current Ambiguities To Keep Explicit

- `config set` stdin mode: preserved in Python and implemented in Rust baseline.
- Python exposes more subcommands (`unset`, `clear`, `load`, `reload`, `export`) than current Rust parity scope for this batch.
- Python includes richer runtime metadata in some payload modes depending on execution policy.
