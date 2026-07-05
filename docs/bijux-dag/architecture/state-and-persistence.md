---
title: State and Persistence
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-05
---

# State and Persistence

Persistence in DAG is not incidental. Run directories, node traces, artifact
indices, and lineage links are the evidence substrate for inspect/replay/diff.

## Visual Summary

```mermaid
flowchart TD
    run["run execution"] --> run_dir["run directory"]
    run_dir --> manifest["manifest and outputs index"]
    run_dir --> node_traces["node traces stdout stderr"]
    manifest --> lineage["artifact lineage and provenance"]
    lineage --> inspect["inspect replay diff consumers"]
```

## Persisted Surfaces

- run manifest and run metadata
- node-level outputs, logs, and traces
- outputs/input index files
- artifact integrity and provenance records
- replay and diff proof-relevant metadata

## Timed-Out Run Evidence

Run-level deadlines now persist their own evidence instead of collapsing into a
generic failed run.

- `manifest.json.status` becomes `timed_out` when the DAG-level deadline is the
  terminal cause.
- `manifest.json.run_timeout_behavior` records whether the runtime finished
  already-running nodes or actively cancelled them at the deadline.
- `.run-incomplete.json` remains present for timed-out runs so partial outputs
  are never misrepresented as fully completed evidence.
- `.run-complete.json` is only written for runs that actually finalized as
  complete.

The two supported timeout behaviors are:

- `finish_running`
  The scheduler stops launching new nodes after the deadline and lets already
  started work finish naturally.
- `cancel_running`
  The scheduler stops launching new nodes and caps in-flight execution to the
  remaining run budget so timeout-capable adapters are terminated at the
  deadline.

## Node Trace Lifecycle Evidence

Each persisted `trace.json` now carries two lifecycle-specific fields in
addition to the coarse terminal `status`.

- `lifecycle_state` records the final runtime state that best matches what
  actually happened to the node.
- `lifecycle_transitions` records the validated state path the runtime observed
  while scheduling or executing that node.

This separation matters because terminal status alone is not always honest
enough. A node can end with status `failed` while its lifecycle state is
`timed_out` or `cancelled`, and a cached node should never claim that execution
started just because it was scheduled for cache lookup.

The persisted lifecycle states are:

- `pending`
- `eligible`
- `queued`
- `running`
- `success`
- `failed`
- `skipped`
- `cached`
- `cancelled`
- `timed_out`

## Code Anchors

- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-artifacts/src/storage/hardening.rs`
- `crates/bijux-dag-artifacts/src/lifecycle/lineage.rs`
- `crates/bijux-dag-runtime/src/artifacts/`
- `crates/bijux-dag-app/src/inspect/run_views.rs`

## Next Reads

- [Artifact Contracts](../interfaces/artifact-contracts.md)
- [Observability and Diagnostics](../operations/observability-and-diagnostics.md)
- [Known Limitations](../quality/known-limitations.md)
