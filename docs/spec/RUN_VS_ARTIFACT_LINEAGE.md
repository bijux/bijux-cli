# Run Lineage vs Artifact Lineage

Run lineage and artifact lineage are related but not interchangeable.

## Run Lineage

Run lineage explains **which run came from which run**.

Primary fields:
- `run_id`: immutable identity of the finalized run directory.
- `run_metadata.parent_run_id`: immediate replay/import parent in run ancestry.
- `run_metadata.source_run_id`: authoritative source run used for replay or import.

Use run lineage for:
- replay ancestry and run-tree navigation,
- run history and operator timeline context,
- provenance of run-level verification reports.

## Artifact Lineage

Artifact lineage explains **which node/run produced which artifact**.

Primary fields:
- artifact identity (`sha256`, artifact id, logical output path),
- producer (`run_id`, `node_id`),
- upstream/downstream artifact edges.

Use artifact lineage for:
- trace-artifact and artifact-inspect workflows,
- retention and GC safety decisions,
- semantic diff and replay mismatch root-cause analysis.

## Boundary Rule

Run lineage must never be used as a substitute for artifact lineage, and artifact lineage must never be treated as run ancestry. The two surfaces are queried together but validated independently.
