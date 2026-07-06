---
title: Distributed Coordination Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Distributed Coordination Model

`bijux-dag` documents simulated distributed coordination semantics without
claiming a production multi-controller scheduler or a remotely authoritative
runtime in `v0.4.0`.

## Scope

This model covers the typed distributed coordination records in
`crates/bijux-dag-runtime/src/backend/distributed/distributed.rs`, the control
plane reconciliation helpers in
`crates/bijux-dag-runtime/src/internal/control/runtime_controls.rs`, and the
status-event proof in
`crates/bijux-dag-runtime/tests/distributed_event_reconciliation_contracts.rs`.

## Current deployment shape

- `single-controller` is the only authoritative execution topology in the
  current release line
- local runtime state remains the source of truth for scheduling, retry
  lineage, and terminal run state
- remote workers, worker pools, and status streams are modeled as typed
  contracts and simulation inputs, not as release-grade runtime ownership

## Single-writer rule

The controller is the only writer allowed to finalize run state and artifact
visibility. Remote workers may execute commands, emit status events, upload
artifacts, and stream logs, but they do not become the authoritative owner of
run completion.

This Single-writer rule exists so duplicate delivery, out-of-order events,
controller restart, and partial remote failure cannot create two conflicting
terminal truths for the same node.

## Event reconciliation contract

Distributed status handling must preserve these invariants:

- out-of-order status events must not revert a newer node state
- duplicate status events must be idempotent
- missing completion events must keep the node non-terminal
- once a terminal event is accepted, later non-terminal snapshots are ignored
- controller restart may reconcile partial remote state, but it must not
  rewrite an already accepted terminal result

## Planner, scheduler, and storage contracts

Distributed modeling must stay subordinate to planner, scheduler, and storage
contracts that already govern the local runtime:

- planner contracts decide the node graph, retry lineage, and dispatch identity
- scheduler contracts decide which node is eligible to run and when a lease may
  be recovered or reassigned
- storage contracts decide when artifacts, traces, and manifests become durable
  evidence

Any future distributed implementation must promote these contracts together.
It is not acceptable to add remote coordination without preserving the existing
planner, scheduler, and storage contracts.

## Not implemented boundary

`v0.4.0` does not ship:

- a production multi-controller scheduler
- cross-host consensus for authoritative run state
- a worker-owned commit path that bypasses controller artifact validation
- release-grade guarantees for network partitions, remote durability, or
  leader-election recovery

These are modeled boundaries only and remain outside the implemented product
surface until code, tests, docs, and release framing are promoted together.

## Related tests

- `crates/bijux-dag-runtime/tests/distributed_contracts.rs`
- `crates/bijux-dag-runtime/tests/distributed_event_reconciliation_contracts.rs`

## Versioning and change policy

Any incompatible change to distributed event ordering, lease recovery,
controller ownership, or artifact commit authority must update this model and
the linked runtime tests in the same change.
