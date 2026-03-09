# Graph Bundle Format v1

## Identifier

`graph-bundle/v1`

## Required fields

- `bundle_version`: `export-bundle/v0.1`
- `format`: `graph-bundle/v1`
- `graph_snapshot`: canonical DAG snapshot
- `manifest`: export manifest metadata

## Optional fields

- `provenance`
- `notes`

## Invariants

- `graph_snapshot.spec` must be parseable as supported DAG schema.
- Graph identity derived from `graph_snapshot` must remain canonical across import/export.
