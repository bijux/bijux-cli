# Run Identity Fields Report

Date: 2026-03-08

## Identity-affecting run fields

- `manifest.run_id`
- `manifest.spec`
- `manifest.graph_fingerprint`
- `manifest.graph_snapshot`
- `manifest.run_metadata.parent_run_id`
- `manifest.run_metadata.source_run_id`

## Identity explanation output fields

- `run_id`
- `run_dir`
- `exists`
- `manifest_exists`
- `created_unix_ms`
- `started_unix_ms`
- `finished_unix_ms`
- `submission_source`
- `trigger_source`
- `parent_run_id`
- `source_run_id`
- `immutability_contract`

## Advisory run fields

- timing fields (`created_unix_ms`, `started_unix_ms`, `finished_unix_ms`)
- summary counters (`run_summary.*`, `node_counts.*`)
- operator labels and trigger metadata
- cache mode and cache directory hints

## Notes

Advisory fields must not be treated as authoritative ancestry identity.
