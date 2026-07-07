---
title: Controller Backend Artifact Boundary
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Controller Backend Artifact Boundary

`bijux-dag` separates controller decisions, backend execution, and artifact
publication so remote or simulated backends cannot silently become the source
of truth.

## Boundary split

- controller owns dispatch identity, retry lineage, lease recovery, and
  terminal state acceptance
- backend workers own command execution, in-flight status emission, and
  provisional artifact upload
- artifact publication becomes authoritative only after the controller accepts
  the result and commits the durable run record

## Artifact handoff rules

- uploads may be staged remotely, but commit authority stays with the
  controller
- logs and traces may be streamed or forwarded, but they remain observational
  until attached to a controller-accepted run record
- checksum and integrity requirements must remain explicit for any remote
  artifact handoff

## Why this boundary exists

Without this split, a partial remote success could publish artifacts or terminal
status that the local runtime never accepted. The boundary keeps replay,
inspection, and evidence generation tied to one durable authority path.

## Primary proof

- `crates/bijux-dag-runtime/tests/distributed_contracts.rs`
- `crates/bijux-dag-runtime/tests/distributed_event_reconciliation_contracts.rs`
- `docs/spec/DISTRIBUTED_COORDINATION_MODEL.md`
