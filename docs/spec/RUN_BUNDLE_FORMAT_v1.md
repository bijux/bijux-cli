# Run Bundle Format v1

## Identifier

`run-bundle/v1`

## Required fields

- `bundle_version`: `export-bundle/v0.1`
- `format`: `run-bundle/v1`
- `manifest`
- `graph_snapshot`
- `node_traces`
- `outputs`

## Optional fields

- `files`
- `provenance`

## Invariants

- Run bundle import must preserve run ancestry/provenance fields when present.
- `node_traces` keys must match `node_id` inside each trace payload.
