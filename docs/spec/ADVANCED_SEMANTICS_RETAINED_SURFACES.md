# Advanced Semantics Retained Surfaces

This page lists advanced semantics families retained in `bijux-dag` and why they remain inside runtime scope.

## Retained families

- `kernel-relevant`
  - Example: `runtime_core/execution/run_state.rs`
  - Reason: controls deterministic terminal-state semantics and replay-safety boundaries.

- `runtime-relevant`
  - Example: `runtime_core/execution/scheduler.rs`
  - Reason: required for runtime scheduling invariants and execution-policy behavior.

- `adapter-relevant`
  - Example: `adapters/registry.rs`
  - Reason: required for adapter resolution determinism and capability contract selection.

## Retention criteria

- Has concrete user-facing runtime path.
- Has direct tests and fixture-backed evidence.
- Has explicit owner (`owner_repo: bijux-dag`) in governance policy.
