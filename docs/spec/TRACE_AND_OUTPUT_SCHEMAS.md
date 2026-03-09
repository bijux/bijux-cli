# TRACE AND OUTPUT SCHEMAS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/EXECUTION_TRACE_RECORDS_CONTRACT.md
# Execution Trace Records Contract

## Purpose

Define required execution trace record semantics for deterministic ordering, completeness, persistence, integrity, and replay inspection.

## Required trace record classes

- node start and node completion events
- scheduler decision events
- artifact read and artifact write events
- replay and cache decision events
- backend dispatch and worker communication events

## Required quality guarantees

- deterministic trace ordering under identical executions
- complete trace coverage for successful, failed, and cancelled runs
- persistence guarantees across runtime restarts
- schema-stable trace serialization
- corruption detection and replay inspection support

## Required governance artifacts

- execution trace regression corpus
- execution trace verification suite
- execution trace benchmark report
- execution trace regression fixtures report

## SOURCE: docs/spec/NODE_STATE_MACHINE.md
# Node state machine

States:
- `queued`
- `ready`
- `running`
- `succeeded`
- `failed`
- `cached`
- `skipped`
- `cancelled`

Legal transitions:
- `queued -> ready`
- `ready -> running`
- `running -> succeeded|failed|cached|cancelled`
- `ready -> skipped|cancelled`
- `queued -> cancelled`

Illegal transitions are rejected by contract tests.

## SOURCE: docs/spec/OUTPUT_CONCISION_CONTRACT.md
# Output Concision Contract

Default output mode is concise and operator-scannable.

Rules:

- human output defaults to short summaries
- machine-readable detail uses `--json`
- diagnostics and proofs remain complete in JSON mode even when human output is compact


## SOURCE: docs/spec/STATE_MACHINE_CONTRACT.md
# State machine contract

## Scope

Defines formal node and run state machines, legal transitions, invariant IDs, and consistency verification behavior.

## Node states

- pending
- eligible
- queued
- running
- success
- failed
- skipped
- cached
- cancelled

## Run states

- submitted
- planning
- running
- paused
- interrupted
- cancelling
- cancelled
- failed
- succeeded

## Invariant IDs

- node transition invariants: `INV-NODE-TRANSITION-*`
- node terminal no-revert invariant: `INV-NODE-TERMINAL-REVERT-001`
- run transition invariants: `INV-RUN-TRANSITION-*`
- failed run causal invariant: `INV-RUN-FAILED-CAUSAL-001`

## Transition guards

Illegal transitions must fail loudly in debug and test paths via transition validation functions.

## Post-run consistency checks

`verify_post_run_state_consistency` validates:

- terminal run has terminal node states
- cancelled run includes cancelled nodes
- failed run contains at least one causal failure

## Operator inspection

`bijux-dev-dag dag verify-state --run-dir <path>` checks state coherence from run artifacts.

## SOURCE: docs/spec/STATE_MACHINE_VISUALIZATION.md
# State machine visualization

## Node transitions

`pending -> eligible -> queued -> running -> (success|failed|cached|cancelled)`

`eligible|queued -> skipped`

## Run transitions

`submitted -> planning -> running -> (succeeded|failed|cancelling)`

`running -> paused -> running`

`running -> interrupted -> (running|cancelling)`

`cancelling -> cancelled`

## SOURCE: docs/spec/TRACE_CONTRACT.md
# Trace Contract

## Scope
Defines trace event ordering, required fields, optional fields, and compatibility constraints.

## Invariants
- Event ordering per node is deterministic for equivalent runs.
- Required fields are always present for persisted events.
- Optional fields are additive and must not break consumers.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`

## Related schemas
- `configs/schema/node_trace.schema.json`

## Versioning and change policy
Breaking event-shape changes require schema versioning and migration plan.

## SOURCE: docs/spec/appendices/runtime/OUTPUT_CONCISION_CONTRACT.md
# Output Concision Contract

Default output mode is concise and operator-scannable.

Rules:

- human output defaults to short summaries
- machine-readable detail uses `--json`
- diagnostics and proofs remain complete in JSON mode even when human output is compact

