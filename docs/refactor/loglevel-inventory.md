# LogLevel Inventory (debug-related identifiers)

Scope: initial pass for replacing debug-only toggles with `LogLevel`.

## `debug`

Where it appears
- CLI command options and config resolution (`src/bijux_cli/cli/commands/*`, `src/bijux_cli/cli/core/output.py`).
- Diagnostics/logging helpers (`src/bijux_cli/cli/core/emit.py`, `src/bijux_cli/infra/emitter.py`).
- Observability/telemetry logging levels (`src/bijux_cli/services/logging/observability.py`, `src/bijux_cli/infra/telemetry.py`).
- Tests and docs referencing `--log-level debug` / `--log-level debug`.

Why it exists
- Historically used as a boolean toggle to enable internal diagnostics and pretty output.

Replacement with LogLevel
- Replace boolean `debug` with `log_level` comparisons: use `LogLevel.DEBUG` (or lower) as the threshold for diagnostics.

## `emit_debug_message`

Where it appears
- `src/bijux_cli/cli/core/emit.py` and call sites (e.g., `src/bijux_cli/cli/commands/status.py`).

Why it exists
- A convenience wrapper that emits internal diagnostics when enabled.

Replacement with LogLevel
- Remove the helper; emit internal diagnostics via `Emitter.emit(..., level=LogLevel.DEBUG)` based on `LogPolicy` thresholds.
