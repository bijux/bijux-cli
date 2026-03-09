# Artifact Identity

## Purpose
Define artifact identity derivation, hash classification rules, and lineage guarantees.

## Context
Artifact identity is the portable output identity surface for comparison and audit.

## Explanation
Artifact identity definition:
- artifact identity is a deterministic token derived from canonical artifact content and identity context.

Identity input domains:
- canonical artifact bytes (or canonical structured representation).
- artifact kind/type.
- optional scoped namespace inputs when required by compatibility policy.

Lineage binding:
- artifact identity must be associated with producing `run_id` and `node_id`.
- lineage binding enables contextual interpretation without mutating artifact hash value.

Classification rules:
- equal artifact identity implies canonical content equivalence under same identity policy version.
- non-equal identity implies content or identity-input divergence.
- unknown/unclassified states must be surfaced explicitly when identity inputs are incomplete.

Algorithm governance:
- hashing algorithm family must be explicitly versioned.
- algorithm migration requires compatibility strategy and documentation.

## Examples
```text
Equivalent artifact content:
source hash: sha256:2f67...
target hash: sha256:2f67...
artifact identity: equal
```

```text
Different canonical content:
source hash: sha256:2f67...
target hash: sha256:8ad1...
artifact identity: drift
```

## Guarantees
- Artifact identity is deterministic for equivalent canonical artifact content.
- Identity drift is explicit and machine-comparable.
- Artifact identity remains lineage-attributable via run and node links.

## Limitations
- Equal identity does not guarantee equivalent external side effects.
- Identity depends on canonicalization correctness for each artifact kind.
- This document does not mandate one universal storage backend.

## Related
- `docs/06-specification/03-artifact-model.md`
- `docs/06-specification/05-run-identity.md`
- `docs/06-specification/08-diff-semantics.md`
- `docs/03-user-guide/03-artifacts.md`
