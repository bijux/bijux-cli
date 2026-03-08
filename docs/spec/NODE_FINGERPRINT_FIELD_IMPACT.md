# Node Fingerprint Field Impact

This mapping documents node-level fields that contribute to node fingerprinting.

## Included in node fingerprint

- `id`
- `kind`
- `inputs` (sorted)
- `outputs.name`
- `outputs.path` (path-normalized)
- `params` (object-key normalized)
- `container`
- `timeout_ms`
- `resources` (normalized defaults)
- `retry`
- `effects` (sorted)
- `env_allowlist` (sorted)
- `tags` (sorted)
- `group`

## Excluded from node fingerprint

- adapter runtime metadata
- run/provenance metadata
- artifact storage metadata
