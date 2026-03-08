# Runtime Engine, State-Machine, and Scheduler Fast Suite

Generated: 2026-03-08

## Suite Membership

- `crates/bijux-dag-runtime/tests/runtime_scheduler_state_machine_invariants_contracts.rs`
- `crates/bijux-dag-runtime/tests/runtime_execution_resilience_contracts.rs`
- `crates/bijux-dag-runtime/tests/state_machine_transitions.rs`
- `crates/bijux-dag-runtime/tests/scheduler_contract.rs`

## Promotion Rule

- A test belongs in the fast suite when:
  - it is deterministic,
  - it has no network dependency,
  - it does not require external adapter binaries,
  - and it runs under one second on baseline CI hardware.

## Coverage Intent

- Engine helper boundaries.
- Scheduler readiness and budget invariants.
- Node/run state-machine legal edge enforcement.
- Recovery behavior for interrupted runs.
