---
title: Local Only Vs Remote Coordinated Runtime
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Local Only Vs Remote Coordinated Runtime

See [Known Limitations](../quality/known-limitations.md) for constraints that
must remain visible when evaluating either execution boundary.

The current `bijux-dag` runtime is a local controller runtime with modeled
remote coordination contracts layered around it.

## Local Runtime Today

- one controller process owns scheduler state, retry policy, and run mutation
- retained run directories remain the authoritative recovery and inspection
  substrate
- cache, replay, and artifact durability are proven through locally governed
  evidence surfaces

## Remote Coordination Today

- worker payloads, heartbeats, leases, remote logs, and status streams are
  typed contracts and proof surfaces
- remote execution semantics support reasoning about duplicate events,
  controller restart, lease recovery, and staged artifact handoff
- these modeled or gated surfaces do not upgrade the stable release into a
  distributed scheduler or public remote-worker product lane

## Reading Rule

When repository docs mention remote or distributed execution, read that as a
coordination boundary under test unless the release boundary explicitly says
the surface is implemented and stable.

## Proof Surfaces

- `docs/spec/DISTRIBUTED_COORDINATION_MODEL.md`
- `docs/spec/REMOTE_EXECUTION_MODEL.md`
- `crates/bijux-dag-runtime/tests/distributed_event_reconciliation_contracts.rs`

## Detailed Walkthrough

Use [Reference: Local Only Vs Remote Coordinated Runtime](local-only-vs-remote-coordinated-runtime.md)
for the narrower comparison between local control and modeled remote lanes.
