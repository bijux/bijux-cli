---
title: Controller Backend Artifact Boundary
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Controller Backend Artifact Boundary

`bijux-dag` keeps controller decisions, backend execution, and artifact
publication on separate authority lines so remote or simulated workers cannot
quietly become the source of truth.

## Boundary Split

- the controller owns dispatch identity, retry lineage, terminal state
  acceptance, and retained run mutation
- backends own command execution, in-flight status emission, and provisional
  artifact production
- artifact visibility becomes authoritative only after controller acceptance
  and durable record commit

## Why This Boundary Exists

Without this split, a partial remote success could publish logs, outputs, or
terminal status that the runtime never accepted as durable evidence.

The boundary keeps replay, inspect, and failure analysis tied to one accepted
run record instead of whichever worker reported first.

## Proof Surfaces

- `docs/spec/DISTRIBUTED_COORDINATION_MODEL.md`
- `docs/spec/REMOTE_EXECUTION_MODEL.md`
- `crates/bijux-dag-runtime/tests/distributed_event_reconciliation_contracts.rs`

## Detailed Walkthrough

Use [Reference: Controller Backend Artifact Boundary](reference/controller-backend-artifact-boundary.md)
for the lower-level handoff rules and examples.
