# Run Manifest Schema v0.1

Source of truth schema: `configs/schema/run_manifest.schema.json`.

Required keys:
- `run_id` (string)
- `created_unix_ms` (number)
- `started_unix_ms` (number)
- `finished_unix_ms` (number)
- `graph_snapshot` (string)
- `graph_fingerprint` (string)
- `status` (string: success|failed|cancelled)
- `spec` (string)
- `tool_version` (string)
- `jobs` (number)
- `adapters` (array)
- `node_counts` (object)
- `policy` (object)

Optional:
- `run_timeout_ms` (number)
- `cache_mode` (string)
- `cache_dir` (string)
- `outputs` (array)
