---
title: State Machine Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# State Machine Contract

`bijux-dag-runtime` treats node and run lifecycle transitions as explicit,
validated contract surfaces.

## Scope

This contract covers the runtime lifecycle model implemented in
`crates/bijux-dag-runtime/src/runtime_core/execution/run_state.rs` and the
operator-facing verification surface exposed by `run_dag_verify_state`.

The contract is about state legality and post-run coherence. It does not claim
that a run has completed successfully just because a trace file exists.

## Node states

Stable node states are:

- pending
- eligible
- queued
- running
- success
- failed
- skipped
- cached
- cancelled
- timed_out

Node transitions must be validated before state changes are treated as durable
runtime truth.

## Run states

Stable run states are:

- submitted
- planning
- running
- paused
- interrupted
- cancelling
- cancelled
- failed
- succeeded
- timed_out

Run transitions must reflect real runtime progression rather than inferred or
synthetic summaries.

## Transition invariants

The lifecycle model is governed by explicit invariant identifiers instead of
implicit comments.

- `INV-NODE-TRANSITION-*` names the legal non-terminal node progression family
- `INV-NODE-TERMINAL-REVERT-001` forbids reverting terminal node states back to
  active execution states
- `INV-RUN-TRANSITION-*` names the legal run progression family
- `INV-RUN-FAILED-CAUSAL-001` requires a failed run to report at least one
  causal node failure

## Allowed node progression

The canonical node progression is:

1. `pending -> eligible`
2. `eligible -> queued`
3. `queued -> running`
4. `running -> success | failed | cancelled | timed_out`

Policy and replay semantics also allow terminal outcomes before execution:

- `pending -> skipped`
- `pending -> cancelled`
- `pending -> timed_out`
- `eligible -> skipped | cached | cancelled | timed_out`
- `queued -> skipped | cached | failed | cancelled | timed_out`

Once a node reaches a terminal state, it must not transition back to a
non-terminal state.

## Allowed run progression

The canonical run progression is:

1. `submitted -> planning`
2. `planning -> running`
3. `running -> paused | interrupted | cancelling | failed | succeeded | timed_out`
4. `paused -> running`
5. `interrupted -> running | cancelling`
6. `cancelling -> cancelled`

No terminal run may transition back into planning or execution.

## Post-run consistency

`verify_post_run_state_consistency` is the proof surface for whole-run
coherence.

- a cancelled run must include at least one cancelled node
- a failed run must report a causal failure count
- terminal runs may not contain non-terminal node states
- imported and replay-derived runs must stay distinguishable from native runs

## Audit and verification surfaces

The runtime exposes stable lifecycle proof surfaces:

- `validate_node_transition`
- `validate_run_transition`
- `verify_post_run_state_consistency`
- `terminal_transition_audit_events`
- `run_dag_verify_state`

These surfaces must move together with any lifecycle contract change.

## Related tests

- `crates/bijux-dag-runtime/tests/state_machine_transitions.rs`
- `crates/bijux-dag-runtime/tests/state_machine_contracts.rs`
- `crates/bijux-dag-runtime/tests/runtime_state_machine_contracts.rs`
- `crates/bijux-dev/tests/state_machine_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to node states, run states, invariant identifiers, or
whole-run consistency rules must update this contract, the linked runtime tests,
and the maintainer hardening guard in the same change.
