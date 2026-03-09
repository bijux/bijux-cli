# Run Directory

Run directory is the authoritative on-disk evidence envelope for one run identity.

## Why this surface exists

Replay, inspect, and diff need stable, attributable evidence after execution completes. Run directory provides that evidence boundary.

## Semantics, not just layout

A run directory records three classes of data:

- authoritative run facts,
- supporting diagnostics,
- derived convenience artifacts.

If these classes are mixed or overwritten, post-run trust degrades.

## Example layout with meaning

```text
runs/RUN_20260309_220/
  run-metadata.json
  node-outcomes/
    extract.json
    transform.json
  artifacts-index.json
  diagnostics/
    transform.stderr.log
  replay/
    replay-summary.json
```

Field semantics:

- `run-metadata.json`: run identity, graph identity, terminal status, timing envelope.
- `node-outcomes/*`: normalized per-node terminal outcomes and reason classes.
- `artifacts-index.json`: artifact IDs, producer-node links, identity references.
- `diagnostics/*`: backend/runtime diagnostic payloads for debug follow-up.
- `replay/*`: replay evidence linked to this run context.

## Authoritative versus derived files

Authoritative:

- run metadata,
- node outcomes,
- artifact linkage/identity references.

Derived:

- cached summaries,
- trend rollups,
- regenerated comparison views.

Rule: if authoritative and derived disagree, authoritative wins and derived must be rebuilt.

## What replay, inspect, and diff require

- inspect requires authoritative run metadata and node outcomes,
- replay requires baseline run identity context and required evidence references,
- diff requires scope-appropriate identity and outcome records.

Missing these required records forces `incomplete`/unknown classifications, not silent assumptions.

## Next reading

- Execution write path: [Execution Engine](../05-system-architecture/03-execution-engine.md)
- Formal run contract: [Run Model Specification](../06-specification/02-run-model.md)
- Debug interpretation path: [Inspect And Debug](../03-user-guide/07-inspect-and-debug.md)
