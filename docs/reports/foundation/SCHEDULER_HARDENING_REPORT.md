# Scheduler Hardening Report

## Purpose

This report records the repository surfaces that currently harden scheduler
determinism, state transitions, and operator-facing scheduler evidence.

## Guarded surfaces

- contract: `docs/spec/SCHEDULER_CONTRACT.md`
- state transitions: `docs/spec/SCHEDULER_STATE_TRANSITIONS.md`
- runtime implementation: `crates/bijux-dag-runtime/src/runtime_core/execution/scheduler.rs`
- runtime tests: `crates/bijux-dag-runtime/tests/scheduler_contract.rs`
- determinism tests: `crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs`
- maintainer guard: `crates/bijux-dev/tests/scheduler_hardening_contracts.rs`
- stable operator route: `bijux-dag runs scheduler-checkpoint`
- command surface: `crates/bijux-dev/src/commands/mod.rs` via `run_dag_scheduler_timeline`

## Current hardening stance

- scheduler determinism is defined through `scheduler_contract_profile()` and
  `deterministic_schedule_order`
- readiness accounting must remain valid across success, cached, skipped,
  failed, and retry-requeue transitions
- budget blocking must stay explicit through `blocked_by_budget` and
  `blocked_reasons`
- checkpoint evidence must stay explicit through `ready_queue`, `scheduled`,
  `inflight`, `completed_statuses`, and `decision_reason`
- operator-facing scheduler evidence must remain available through
  `scheduler.checkpoint.json`, `bijux-dag runs scheduler-checkpoint`,
  `observability.timeline.json`, and `run_dag_scheduler_timeline`
