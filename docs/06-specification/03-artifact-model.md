# Artifact Model

## Purpose
Define artifact entity semantics, lineage requirements, and portability constraints.

## Context
Artifacts are the durable output contract connecting execution, inspection, replay, and diff.

## Explanation
Artifact entity fields:
- `artifact.id`: identity-derived artifact key.
- `artifact.run_id`: producing run identity.
- `artifact.node_id`: producing node identifier.
- `artifact.path` or logical location descriptor.
- `artifact.kind`: classification (file/directory/structured output).
- `artifact.hash`: content-derived checksum.
- `artifact.metadata`: optional non-semantic annotations.

Artifact creation rules:
- an artifact is attributable to one producing node result in one run.
- artifact content hash is computed over canonical bytes for that artifact kind.
- lineage links to `run_id` and `node_id` are mandatory for auditable provenance.

Artifact hashing algorithm contract:
- hashing algorithm family must be stable for a compatibility window.
- algorithm changes require versioned migration policy and compatibility handling.
- hash computation must be deterministic for equivalent canonical content.

Portability rules:
- export/import workflows preserve artifact identity context and lineage metadata.
- portability validation is determined through replay and diff, not by transport success alone.

## Examples
```text
Artifact lineage example:
artifact.id: a_7b4...
produced_by: run r_9f1... / node test
hash: sha256:2f67...
```

```text
Artifact portability check:
source artifact hash == target artifact hash
-> classified as equivalent output for that artifact unit
```

## Guarantees
- Every durable artifact has explicit producer lineage.
- Artifact identity is hash-backed and deterministic for equivalent content.
- Artifact contract supports replay/diff comparability across runs.

## Limitations
- This model does not require all nodes to emit artifacts.
- External system side effects are not automatically captured as artifacts.
- Physical storage backend implementation is outside this contract.

## Related
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/06-artifact-identity.md`
- `docs/06-specification/08-diff-semantics.md`
- `docs/05-system-architecture/06-artifact-store.md`
