# Internal Invariants Debugging Report

## Debugging surfaces

- scheduler debug event log: `scheduler_debug_event_log` in `runtime_core/execution/scheduler.rs`
- invariant registry and ids: `runtime_core/governance/invariants.rs`
- run and node transition invariant ids: `runtime_core/execution/run_state.rs`

## Debugging expectations

- invariant failures are diagnosable through stable invariant ids
- trace-capture and debug event logs preserve ordering and causality
- invariant failure simulation remains executable in runtime contract tests
