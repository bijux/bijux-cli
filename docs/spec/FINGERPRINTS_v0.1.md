# Fingerprints v0.1

## Graph Fingerprint
- Compute canonical JSON for the entire graph.
- Hash with SHA256 of the UTF-8 bytes.

## Node Fingerprint
- Compute canonical JSON for the node only.
- Use resolved params (graph inputs substituted; output refs remain explicit).
- Hash with SHA256 of the UTF-8 bytes.

## Exclusions
- Runtime-only fields such as timestamps or execution status are excluded.
