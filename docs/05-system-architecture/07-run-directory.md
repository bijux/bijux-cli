# Run Directory

Describe run directory architecture and its role in execution evidence lifecycle.

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

## Run-directory semantics in practice

Run directory is the authoritative evidence envelope for one run identity. It is not just a file tree; it is the persisted execution narrative used by inspect, replay, and diff.

## Realistic run-directory tree example

```text
runs/RUN_20260309_220/
  run-metadata.json
  node-outcomes/
    extract.json
    transform_orders.json
    publish.json
  artifacts-index.json
  replay/
    replay-summary.json
  diagnostics/
    stderr-transform_orders.log
```

What each record means:

- `run-metadata.json`: run identity, graph identity, terminal status, timing envelope.
- `node-outcomes/*.json`: per-node normalized outcomes and reason codes.
- `artifacts-index.json`: artifact IDs, hashes, producer-node linkage.
- `replay/replay-summary.json`: replay classification evidence when replay exists.
- `diagnostics/*`: backend diagnostics referenced by inspect workflows.

## Authoritative versus derived data

Authoritative run records:

- run metadata,
- node terminal outcomes,
- artifact identity/linkage references.

Derived views:

- summarized dashboards,
- trend indexes,
- cached comparison artifacts.

If authoritative and derived views diverge, trust authoritative run-directory records and regenerate derived views.
