---
title: Scheduler Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Scheduler Contract

`bijux-dag-runtime` owns deterministic ready-node selection, budget-aware batch
formation, retry requeue semantics, and scheduler-facing event evidence for a
single DAG run.

## Scope

This contract covers the runtime scheduler that operates on an execution plan
after planning and validation have completed.

It does not promise a distributed scheduler service, a remote queueing system,
or a long-running control plane. Public operator documentation may describe
schedule and backfill workflows, but the invariant contract here is the
in-process runtime scheduler plus its proof surfaces.

## Canonical scheduling model

The stable scheduler profile is exposed by `scheduler_contract_profile()` and
must remain reviewable in code, tests, and docs.

- canonical unit: `Node`
- model: `EventDriven`
- priority model: `StaticHints`
- ready tie break: `PriorityCpuMemoryFitThenNodeId`

The runtime may evolve implementation details, but any incompatible change to
the canonical unit, model, priority interpretation, or tie-break semantics is a
contract change.

## Determinism invariants

The scheduler must preserve these invariants for the same plan, policy, and
node readiness sequence:

- `deterministic_schedule_order` returns a stable ordering for equal inputs
- the ready frontier is dependency-correct and deterministic
- tie breaks remain stable through node id ordering after priority and resource
  fit are applied
- concurrency settings change batch width, not plan membership semantics
- `scheduler_invariants_hold` remains true after success, cached, skipped,
  retry, and failure transitions

## Budget and cancellation invariants

Scheduling decisions must stay explicit about why work was or was not
dispatched.

- `max_parallelism` caps batch width
- `cpu_budget`, `memory_budget_mb`, and `gpu_device_budget` may block ready
  nodes without changing their dependency truth
- blocked work must surface through `blocked_by_budget` and `blocked_reasons`
- cancellation returns an empty batch with `cancelled = true`
- run timeout returns an empty batch with `timed_out = true`

## Failure and retry semantics

Failure handling is a scheduler contract, not an incidental executor detail.

- `FailurePropagationMode::FailFast` stops new dispatch after the first node
  failure and records undispatched remainder as aborted propagation
- `FailurePropagationMode::IsolateBranch` lets unrelated subgraphs continue but
  skips every descendant of the failed node with
  `skip_reason.reason = "isolated_branch_failure"`, even when a permissive
  trigger rule such as `all_done` would otherwise release the join
- `FailurePropagationMode::ContinueIndependent` keeps evaluating trigger rules
  from terminal upstream states, so joins and other downstream nodes may still
  run when their workflow semantics allow it
- `queue_retry` records that a node is awaiting replay eligibility
- `requeue_retries` moves queued retry nodes back into the ready frontier
- cached and skipped completions satisfy downstream readiness exactly like their
  declared runtime semantics require

## Failure evidence contract

Failure propagation decisions must stay inspectable after the run ends.

- `failure-propagation.json` records typed propagation entries with node status,
  reason, blocking nodes, and propagation mode
- skipped descendants created by branch isolation use the durable reason
  `isolated_branch_failure`
- replay preserves the recorded propagation decision instead of reclassifying
  the skipped descendant during inspection
- node trace `transition_cause` stays aligned with the propagation reason so
  downstream evidence remains stable across human and JSON views

## Observability proof

The scheduler contract is backed by runtime tests and by operator-facing
timeline evidence.

- `crates/bijux-dag-runtime/tests/scheduler_contract.rs` proves deterministic
  dispatch, readiness accounting, failure propagation, and budget enforcement
- `crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs`
  proves stable ordering for equal inputs
- `scheduler.checkpoint.json` records loop-boundary scheduler state with
  `ready_queue`, `scheduled`, `blocked_by_budget`, `inflight`,
  `completed_statuses`, and `decision_reason`
- `bijux-dag runs scheduler-checkpoint` exposes the retained checkpoint through
  a stable operator route and reports absent or corrupt checkpoint evidence
  explicitly
- `run_dag_scheduler_timeline` exposes scheduler entries from
  `observability.timeline.json`
- scheduler timeline evidence is filtered to scheduler-relevant categories such
  as `schedule`, `dispatch`, `retry`, `cache_hit`, and `cache_miss`

## Related tests

- `crates/bijux-dag-runtime/tests/scheduler_contract.rs`
- `crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs`
- `crates/bijux-dag-runtime/tests/scheduler_ordering_fairness_contracts.rs`
- `crates/bijux-dag-runtime/tests/concurrency_contracts.rs`
- `crates/bijux-dev/tests/scheduler_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to scheduler determinism, budget blocking semantics,
retry requeue behavior, or timeline evidence categories must update this
document, the linked runtime tests, and the maintainer hardening guard in the
same change.
