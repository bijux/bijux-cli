# State Machine Hardening Report

## Purpose

This report records the repository surfaces that currently harden lifecycle
state legality, terminal transition auditability, and operator-facing run-state
verification.

## Guarded surfaces

- contract: `docs/spec/STATE_MACHINE_CONTRACT.md`
- visualization: `docs/spec/STATE_MACHINE_VISUALIZATION.md`
- runtime implementation: `crates/bijux-dag-runtime/src/runtime_core/execution/run_state.rs`
- runtime tests: `crates/bijux-dag-runtime/tests/state_machine_transitions.rs`
- lifecycle contracts: `crates/bijux-dag-runtime/tests/state_machine_contracts.rs`
- manifest coherence: `crates/bijux-dag-runtime/tests/runtime_state_machine_contracts.rs`
- state fixtures:
  - `crates/bijux-dag-runtime/tests/fixtures/state_machine/evolution_trace.json`
  - `crates/bijux-dag-runtime/tests/fixtures/state_machine/cancellation_trace.json`
- maintainer guard: `crates/bijux-dev/tests/state_machine_hardening_contracts.rs`
- command surface: `crates/bijux-dev/src/commands/mod.rs` via `run_dag_verify_state`

## Current hardening stance

- legal node and run transitions must stay explicit through `validate_node_transition`
  and `validate_run_transition`
- terminal states must not silently revert once they are recorded
- whole-run verification must keep failed, cancelled, timed out, and succeeded
  summaries coherent with node outcomes
- operator-facing verification must remain available through
  `verify_post_run_state_consistency` and `run_dag_verify_state`
