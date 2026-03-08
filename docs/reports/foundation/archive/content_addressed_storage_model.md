# Content Addressed Storage Model

Generated from artifact store implementation capability declarations.

## Identity primitives

- `artifact_sha256` identifies content bytes.
- `artifact_id` identifies logical artifact identity (`<node_id>:<file_name>`).
- Durable provenance joins `artifact_sha256` with `run_id`, `node_id`, and `node_fingerprint`.

## Implementation status

- Filesystem backend: implemented read/write payload persistence.
- Object backend: modeled-only surface; runtime rejects read/write calls.

## Safety rules

- Artifact identity is content + provenance; identical bytes can legitimately appear under distinct provenance chains.
- Garbage collection decisions must remain lineage-aware and dry-run explainable.
