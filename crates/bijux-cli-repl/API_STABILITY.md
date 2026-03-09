# REPL API Stability

## Stable surface

These items are intended for cross-crate use and parity harnesses:

- `startup_repl`
- `startup_repl_with_diagnostics`
- `shutdown_repl`
- `configure_history`
- `load_history`
- `flush_history`
- `replay_history_command`
- `register_plugin_completion_hook`
- `completion_candidates`
- `execute_repl_input`
- `execute_repl_line`
- `repl_argv_from_line`
- `inspect_last_error`
- `session_diagnostics_dump`
- `render_repl_command_reference`
- `benchmark_startup_latency`
- `check_repl_budgets`

Stable types:

- `ReplSession`
- `ReplStartupContract`
- `ReplShutdownContract`
- `ReplInput`
- `ReplEvent`
- `ReplFrame`
- `ReplStream`
- `ReplError`

## Internal behavior (not stable)

- Exact command-reference wording beyond snapshot-controlled lines.
- Internal history parser fallback heuristics.
- Internal REPL diagnostics warning text.
- Exact payload shape of placeholder handler responses.

## Compatibility intent

The crate keeps parser/routing/kernel/emitter integration behavior aligned with the shared Rust execution stack, while transcript behavior is tracked against current Python behavior artifacts.
