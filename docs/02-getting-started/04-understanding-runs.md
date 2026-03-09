# Understanding Runs

A run is the primary historical object in bijux-dag. It is the unit you inspect, replay, diff, and audit.

## What a run contains

A run record typically contains:

- `run_id`: unique execution identity,
- `graph_id`: identity of the graph definition executed,
- lifecycle status and timing envelope,
- per-node outcome records,
- artifact references and lineage links.

Why this matters: if you cannot identify and inspect a run, you cannot make reliable equivalence or drift claims.

## Concrete run record example

```json
{
  "run_id": "RUN_20260309_301",
  "graph_id": "ORDERS_FIRST_GRAPH",
  "status": "succeeded",
  "nodes": {
    "extract_orders": {"status": "succeeded"},
    "summarize_orders": {"status": "succeeded"}
  },
  "artifacts": [
    {"artifact_id": "ART_901", "node_id": "extract_orders"},
    {"artifact_id": "ART_902", "node_id": "summarize_orders"}
  ]
}
```

How to read it:

- `run_id` identifies the exact execution instance,
- `graph_id` ties it to definition state,
- `nodes` show outcome surface,
- `artifacts` show output lineage anchors.

## Run classes you must distinguish

- successful run: reaches terminal success with full expected outcomes.
- failed run: terminal failure with diagnosable node-level reason.
- replayed run: new run produced by replay workflow against baseline context.
- imported run: run context materialized from external bundle/provenance source.

Treat these as different evidence classes during incident and release analysis.

## Run identity, graph identity, and ancestry

Relationship model:

- many runs can share one graph identity,
- each run has its own run identity,
- ancestry links (original/replayed/imported) explain provenance lineage.

Common mistake: assuming equal graph identity implies equal run behavior. It does not.

## Next reading

- First-run operational sequence: [Running A Pipeline](../02-getting-started/03-running-a-pipeline.md)
- Practical history interpretation: [Run History](../03-user-guide/04-run-history.md)
- Formal run contract: [Run Model Specification](../06-specification/02-run-model.md)
