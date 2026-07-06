---
title: Concurrency Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Concurrency Model

`bijux-dag-runtime` supports bounded local concurrency inside one run while
preserving deterministic scheduler and artifact invariants.

## Scope

This contract covers runtime coordination and concurrent access patterns inside
the local DAG runtime. It does not promise distributed execution, remote worker
coordination, or cross-host consensus.

## Concurrency guarantees

The runtime must preserve these guarantees under concurrent activity:

- concurrent predecessor completion releases a downstream node at most once
- concurrent trace-write registration keeps summary counters and trace indices
  aligned
- cache fingerprint claims have a single winner
- cancellation, retry, and failure races do not violate scheduler invariants
- timeout and exit classification prefer timeout once the timeout has been
  recorded
- latest-link updates remain a tracked coordination event rather than an
  unowned side effect

## Coordination surfaces

The model is enforced through explicit runtime surfaces:

- `RuntimeCoordinationState`
- `SchedulerState`
- `scheduler_invariants_hold`
- `merge_timeout_and_exit_events`
- `thread_safety_audit`

These surfaces define the proof boundary for local concurrency correctness.

## Operational limits

Concurrency in `bijux-dag` is bounded and local by design.

- scheduler parallelism is constrained by `max_parallelism`
- budget enforcement may further reduce concurrent dispatch
- the bounded local executor queue prevents unbounded submission growth
- active-run guards reject import or export reads that would race with an
  in-progress run

## Related tests

- `crates/bijux-dag-runtime/tests/concurrency_contracts.rs`
- `crates/bijux-dag-runtime/tests/scheduler_contract.rs`
- `crates/bijux-dag-runtime/tests/scheduler_ordering_fairness_contracts.rs`

## Versioning and change policy

Any incompatible change to local concurrency guarantees, coordination surfaces,
or active-run safety behavior must update this contract and the linked runtime
tests in the same change.
