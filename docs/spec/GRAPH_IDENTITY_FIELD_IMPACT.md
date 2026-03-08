# Graph Identity Field Impact

This mapping documents which graph fields affect identity hashing.

## Included in graph identity

- `spec` after alias normalization (`0.1`/`v0.1` -> `bijux-dag/v0.1`)
- `inputs` (with map key sorting)
- `nondeterminism_allowed`
- node fields: `id`, `kind`, `inputs`, `outputs`, `params`, `container`, `timeout_ms`, `resources`, `retry`, `effects`, `env_allowlist`, `tags`, `group`
- edge fields: `from.node_id`, `from.port`, `to.node_id`, `to.port`

## Normalized before hashing

- node order (sorted by `id`)
- edge order (sorted by `from/to` tuple)
- `outputs.path` path separators
- `params` object key order
- `inputs` map key order
- `env_allowlist`, `effects`, `inputs`, `tags` ordering
- `resources` with `{cpu:0, mem_mb:0}` are normalized to `null`

## Excluded from graph identity

- backend adapter/runtime version metadata
- run-level metadata
- artifact-level metadata

## Generated report

Machine-readable decomposition:

- `docs/reports/foundation/graph_identity_decomposition_report.json`
