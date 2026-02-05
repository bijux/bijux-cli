# Fingerprints v0.1

## Graph Fingerprint
- Compute canonical JSON for the entire graph.
- Hash with SHA256 of the UTF-8 bytes.
- Graph metadata is included when present.

## Node Fingerprint
- Compute canonical JSON for the node only.
- Use resolved params (graph inputs substituted; node output refs resolve to declared output paths).
- Hash with SHA256 of the UTF-8 bytes.

## Runtime Node Fingerprint
- Start with the base node fingerprint above.
- Incorporate the materialized inputs index (path + sha256 + provenance) in a stable order.
- Hash with SHA256 of the UTF-8 bytes.

## Exclusions
- Runtime-only fields such as timestamps or execution status are excluded.
- `group` is excluded from node fingerprints.
