# Artifact Model

Define artifact entity semantics, lineage requirements, and portability constraints.

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

Formal artifact rules:
- RULE-ART-001: each artifact MUST have one `artifact.id`.
- RULE-ART-002: each artifact MUST reference producing `run_id` and `node_id`.
- RULE-ART-003: artifact identity MUST be derived from canonical content policy.
- RULE-ART-004: lineage links MUST remain queryable for inspect/diff workflows.

Portability rules:
- export/import workflows preserve artifact identity context and lineage metadata.
- portability validation is determined through replay and diff, not by transport success alone.

Invalid state definitions:
- INVALID-ART-MISSING-ID: artifact identity absent.
- INVALID-ART-MISSING-LINEAGE: missing run or node linkage.
- INVALID-ART-HASH-MISSING: artifact expected to be identity-tracked but hash unavailable.
- INVALID-ART-CANONICALIZATION-UNKNOWN: canonical content basis cannot be determined.

Edge cases:
- zero-byte artifacts are valid if canonicalized and lineage-attributed.
- directory artifacts are valid if canonical directory representation policy is defined.
- partial artifact availability is valid in failed runs but must be explicitly represented.

Compatibility notes:
- hash algorithm upgrades require documented compatibility window and migration path.
- new artifact kinds are compatible when canonicalization and lineage semantics are defined.

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
