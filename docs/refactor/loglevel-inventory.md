# LogLevel Inventory (verbosity-related identifiers)

Scope: initial pass for replacing verbosity/debug booleans with `LogLevel`.

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

## `verbose`

Where it appears
- CLI flags and command payload shaping (`src/bijux_cli/cli/commands/*`).
- Execution policy resolution (`src/bijux_cli/core/precedence.py`).
- Context/DI logging gates (`src/bijux_cli/core/context.py`, `src/bijux_cli/core/di.py`).
- Tests and docs for `--verbose`.

Why it exists
- Historically used as a separate boolean axis for “include runtime metadata.”

Replacement with LogLevel
- Replace `verbose` with `LogLevel` threshold (e.g., include runtime metadata when `log_level <= LogLevel.DEBUG` or a chosen threshold). The policy should use `log_level` only.

## `set_verbose`

Where it appears
- `src/bijux_cli/core/di.py` on `DIContainer`.

Why it exists
- Provides a global DI verbosity flag to decide whether DI operations emit internal logs.

Replacement with LogLevel
- Remove the method; DI internal logging should be controlled via `LogLevel` threshold in `LogPolicy`.

## `_coerce_verbose`

Where it appears
- `src/bijux_cli/core/precedence.py`.

Why it exists
- Normalizes `verbose` flags to a numeric level (`verbose_level`).

Replacement with LogLevel
- Remove when `verbose` is no longer a separate axis; `LogLevel` ordering should drive all verbosity behavior.

## `emit_debug_message`

Where it appears
- `src/bijux_cli/cli/core/emit.py` and call sites (e.g., `src/bijux_cli/cli/commands/status.py`).

Why it exists
- A convenience wrapper that emits internal diagnostics when enabled.

Replacement with LogLevel
- Remove the helper; emit internal diagnostics via `Emitter.emit(..., level=LogLevel.DEBUG)` based on `LogPolicy` thresholds.
