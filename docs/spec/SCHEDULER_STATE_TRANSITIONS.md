---
title: Scheduler State Transitions
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Scheduler State Transitions

`SchedulerState` is the durable runtime model for ready-node evolution inside a
single run.

## State model

The scheduler keeps four pieces of state aligned:

- `indegree`: remaining unsatisfied dependencies per node
- `ready`: deterministic frontier of nodes eligible to schedule
- `retry_queue`: nodes deferred for retry eligibility
- `completion_by_node`: terminal completion class already observed

The state machine is only valid when dependency truth, queue membership, and
completion bookkeeping agree.

## Node lifecycle transitions

The runtime exposes these transition methods on `SchedulerState`:

- `complete_success`
- `complete_cached`
- `complete_skipped`
- `complete_failed`
- `queue_retry`
- `requeue_retries`

Each transition must keep `scheduler_invariants_hold` true for a valid event
sequence.

## Transition rules

| transition | input state | effect |
| --- | --- | --- |
| `complete_success(node)` | node is ready or in flight | removes the node from `ready`, records `success`, decrements downstream indegree, and emits newly ready nodes |
| `complete_cached(node)` | node completed from cache | records `NodeCached`, then applies success-like dependency release semantics |
| `complete_skipped(node)` | node was intentionally skipped | records `NodeSkipped`, then applies success-like dependency release semantics |
| `complete_failed(node, mode)` | node failed | records `NodeFailed`; downstream release depends on `FailurePropagationMode` |
| `queue_retry(node)` | node is awaiting retry policy | inserts the node into `retry_queue` and records `NodeRetryQueued` |
| `requeue_retries()` | retry backoff expired | moves queued retry nodes back into `ready` and records `NodeRetryRequeued` |

## Event evidence

The scheduler event log is part of the proof surface. Stable event kinds are:

- `NodeReady`
- `NodeScheduled`
- `NodeBlockedByBudget`
- `NodeRetryQueued`
- `NodeRetryRequeued`
- `NodeCached`
- `NodeSkipped`
- `NodeFailed`

Sequence numbers must stay monotonic within one scheduler state history.

## Failure propagation

`FailurePropagationMode` decides whether failed work releases downstream nodes.

- `FailFast` preserves failure isolation and releases nothing
- `IsolateBranch` allows downstream readiness when branch isolation semantics
  permit it
- `ContinueIndependent` and `QuorumLikeFuture` remain explicit modes so future
  behavior cannot silently collapse into `FailFast`

## Timeline and checkpoint relationship

`ExecutionCheckpoint` and scheduler timeline output exist to make transitions
auditable after the run ends.

- `ready_queue`, `blocked_by_budget`, and `completed_statuses` capture the
  scheduler snapshot at a loop boundary
- `scheduler_debug_event_log` exposes the internal event history
- `run_dag_scheduler_timeline` turns `observability.timeline.json` into an
  operator-facing scheduler report

## Related code and tests

- `crates/bijux-dag-runtime/src/runtime_core/execution/scheduler.rs`
- `crates/bijux-dag-runtime/tests/scheduler_contract.rs`
- `crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs`
- `crates/bijux-dev/tests/scheduler_hardening_contracts.rs`
