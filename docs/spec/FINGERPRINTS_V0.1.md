# Fingerprints v0.1

`graph_id` is the canonical graph fingerprint identifier.

## Graph Fingerprint
- Compute canonical JSON for the entire graph.
- Hash with SHA256 of the UTF-8 bytes.
- Graph metadata is included when present.
- Exposed as `graph_id`.

Contributes directly:
- `spec`
- `meta.name`, `meta.description`, `meta.owners`, `meta.tags`
- `inputs`
- `nondeterminism_allowed`
- node list (after canonical ordering)
- edge list (after canonical ordering)

## Node Fingerprint
- Compute canonical JSON for the node only.
- Use resolved params (graph inputs substituted; node output refs resolve to declared output paths).
- Hash with SHA256 of the UTF-8 bytes.

Contributes directly:
- `id`
- `kind`
- `inputs`
- `outputs` (with canonical path normalization)
- resolved `params`
- `container`
- `timeout_ms`
- `resources`
- `tags`
- `retry`
- `effects`
- `env_allowlist`

## Runtime Node Fingerprint
- Start with the base node fingerprint above.
- Incorporate the materialized inputs index (path + sha256 + provenance) in a stable order.
- Hash with SHA256 of the UTF-8 bytes.

## Exclusions
- Runtime-only fields such as timestamps or execution status are excluded.
- `group` is excluded from node fingerprints.

## Explain surface

- `dag fingerprint --explain`
- `dag hash graph --explain`
