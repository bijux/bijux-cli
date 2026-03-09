# Artifact Model

Normative contract for artifact payload, metadata, lineage, and corruption handling.

## Terms

- Artifact: persisted output unit with identity and provenance linkage.
- Payload: canonical content basis used for identity.
- Lineage: producing run/node and ancestry references.

## Required fields

- `artifact_id`
- producing `run_id`
- producing `node_id`
- artifact kind/type
- identity/hash reference
- storage/payload reference

## Field classes

Payload fields:

- canonical output content or canonicalized representation.

Metadata fields:

- identity values, kind, timestamps, storage references.

Lineage fields:

- run/node producer linkage,
- ancestry references used by inspect/replay/diff.

RULE-ART-001: these classes MUST remain distinguishable.

## Core rules

- RULE-ART-002: artifact identity MUST derive from canonical payload policy.
- RULE-ART-003: lineage links to producing run/node MUST be present.
- RULE-ART-004: artifact records MUST be queryable for diff/replay workflows.

## Provenance and identity

Artifact identity proves content-equivalence class.
Provenance proves production context.
Both are required for trustworthy interpretation.

## Invalid and corrupt states

- INVALID-ART-MISSING-ID
- INVALID-ART-MISSING-LINEAGE
- INVALID-ART-HASH-MISSING
- INVALID-ART-CANONICALIZATION-UNKNOWN
- CORRUPT-ART-METADATA-PAYLOAD-MISMATCH
- CORRUPT-ART-MISSING-PAYLOAD-REFERENCE

Corrupt artifacts MUST be treated as non-trustworthy evidence until repaired or regenerated.

## Incomplete-state cases

- artifact reference exists but payload unavailable due to retention,
- lineage partially available after import,
- identity present but policy-version compatibility unresolved.

Incomplete states require bounded/unknown comparison handling, not forced equivalence.

## Next reading

- Artifact identity inputs/exclusions: [Artifact Identity](../06-specification/06-artifact-identity.md)
- Portability implications: [Portability](../05-system-architecture/10-portability.md)
