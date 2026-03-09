# Run Directory

## Purpose
Describe run directory architecture and its role in execution evidence lifecycle.

## Context
Run directory design supports inspect, replay, and diff workflows across time.

## Explanation
Run directory responsibilities:
- capture run-level metadata
- preserve node execution outcomes
- provide structured evidence for diagnostics and comparison

Run directory layout guidance (conceptual):
- run metadata record (identity, status, timing envelope).
- node outcome records (per-node terminal outcomes and diagnostics).
- artifact reference index (produced artifact links).
- replay/diff helper records where applicable.

Design principles:
- one run identity maps to one coherent run evidence scope
- file/layout organization should support fast operational lookup
- run evidence remains attributable and auditable

Run directory in workflow:
- created during run initialization
- updated throughout execution
- consumed by inspect/replay/diff surfaces after completion

## Examples
```text
Run start -> directory materialization -> incremental evidence writes -> terminal state snapshot
```

```text
Conceptual layout:
runs/RUN_.../
  run-metadata.json
  node-results.json
  artifacts-index.json
```

## Guarantees
- Run directory is documented as a first-class architecture surface.
- Evidence lifecycle is explicit from initialization to post-run analysis.

## Limitations
- This page does not define exact on-disk schema.
- Retention policy and cleanup strategy are environment-specific.

## Related
- `docs/05-system-architecture/03-execution-engine.md`
- `docs/03-user-guide/04-run-history.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
- `docs/06-specification/02-run-model.md`
