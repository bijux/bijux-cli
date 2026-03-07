# Scheduler Contract

## Scope

This document defines scheduler semantics for bijux-dag runtime execution.
The scheduler contract covers ready-queue behavior, tie-breaking, retries,
cache/skip/failure downstream readiness semantics, and scheduler debug artifacts.

## Canonical Unit

- Canonical scheduling unit: `node`
- Attempt handling is modeled as node lifecycle events with retry requeue.

## Runtime Model

- Scheduler model: `event-driven`
- Ready queue source: dependency indegree transitions to zero
- Tie-breaking for equally ready nodes: lexical order on `node_id`
- Priority model: no dynamic priority class in runtime node scheduler

## Readiness Semantics

- A downstream node becomes ready when all required predecessors are satisfied.
- Satisfaction events:
- `success`
- `cached`
- `skipped`
- Failure propagation modes:
- `fail_fast`: failed predecessor does not satisfy downstream readiness.
- `isolate_branch`: failed predecessor is treated as branch-local and may unlock
  independent downstream behavior.
- `continue_independent`: equivalent downstream readiness behavior to
  `isolate_branch` for current runtime.
- `quorum_like_future`: non-fail-fast behavior reserved for future quorum policy.

## Retry Semantics

- Retry enters `retry_queue` and must be requeued into `ready_queue` explicitly.
- Retry requeue must preserve deterministic tie-breaking.
- A node cannot exist in both `ready_queue` and `retry_queue` simultaneously.

## Scheduler State Model

Runtime exposes a dedicated state type:

- `SchedulerState`
- Owns indegree counters, adjacency, ready queue, retry queue, completion map,
  and scheduler event log.
- Provides explicit completion transitions:
- `complete_success`
- `complete_cached`
- `complete_skipped`
- `complete_failed`

## Event Log Model

Scheduler debug events are structured as:

- `sequence`
- `kind`
- `node_id`
- optional `detail`

Event kinds:

- `node_ready`
- `node_scheduled`
- `node_blocked_by_budget`
- `node_retry_queued`
- `node_retry_requeued`
- `node_cached`
- `node_skipped`
- `node_failed`

## Determinism Requirements

- For a fixed graph and fixed event sequence, `ready_queue` evolution is deterministic.
- A downstream node becomes ready at most once.
- Concurrency level (`jobs`, `max_parallelism`) does not change scheduler semantic node set.

## Invariants

Scheduler invariants required in code and tests:

- Event sequence numbers are unique and monotonically increasing.
- No node can be in both retry queue and ready queue.
- Retry requeue does not duplicate nodes.

## Timeline Artifact

Runtime emits scheduler timeline data in run artifacts. Control plane command:

- `bijux-dev-dag dag scheduler-timeline --run-dir <path>`

This command emits scheduler-relevant timeline entries from
`observability.timeline.json` for completed runs.

## Versioning

- Contract version policy: additive-only for required fields within `v1`.
- Breaking semantic changes require a new major contract version.

## Verifying Tests and Checks

- `crates/bijux-dag-runtime/tests/scheduler_contract.rs`
- `bijux-dev-dag repo run --domain governance` (`scheduler-invariants` suite)
