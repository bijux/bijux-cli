---
title: Local Only Vs Remote Coordinated Runtime
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Local Only Vs Remote Coordinated Runtime

The current `bijux-dag` runtime is implemented as a local controller with
explicitly modeled remote coordination surfaces.

## Local-only runtime today

- one controller process owns scheduler state and run-state mutation
- cache, replay, and artifact durability are validated through local proof
  surfaces
- restart recovery is scoped to locally governed run directories and runtime
  records

## Remote-coordinated surface today

- worker pools, heartbeats, leases, remote logs, and status streams are typed
  contracts and simulated proofs
- remote worker payloads are typed runtime envelopes that carry graph, node,
  params, verified input artifacts, workspace paths, policy, and execution
  fingerprints
- a modeled remote worker can execute `const` and `shell` payloads and returns
  the shared `NodeResult` schema used by local execution
- remote coordination supports reasoning about duplicate events, partitioned
  status delivery, lease recovery, and artifact handoff semantics
- these surfaces do not upgrade the runtime into a distributed scheduler

## Operational reading

When docs or commands mention remote or distributed execution, interpret them
as modeled coordination boundaries unless the release boundary explicitly says
otherwise. The current product promise remains a local controller runtime with
future-facing distributed semantics under test. The controller still owns
scheduler state, retry policy, and run mutation; the modeled worker lane is a
typed execution contract, not a separate production scheduler service.

## Primary proof

- `docs/spec/DISTRIBUTED_COORDINATION_MODEL.md`
- `docs/spec/REMOTE_EXECUTION_MODEL.md`
- `crates/bijux-dag-runtime/tests/distributed_contracts.rs`
- `crates/bijux-dag-runtime/tests/distributed_event_reconciliation_contracts.rs`

## Next Reads

- [Release Boundary](../../foundation/release-boundary.md)
- [Known Limitations](../../quality/known-limitations.md)
