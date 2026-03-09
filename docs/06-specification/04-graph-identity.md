# Graph Identity

## Purpose
Define how graph identity is derived and which inputs are identity-relevant.

## Context
Graph identity anchors replay planning and drift detection at definition level.

## Explanation
Graph identity definition:
- graph identity is a deterministic digest of canonical graph semantics.
- identity domain is definition-level and independent of one specific run attempt.

Identity-relevant inputs:
- normalized node definitions and dependency structure.
- semantic execution configuration declared as definition-level contract inputs.

Identity-irrelevant inputs:
- formatting-only changes.
- comments and non-semantic annotation fields.
- ordering noise normalized by canonicalization rules.

Graph identity derivation pipeline:
1. parse graph definition.
2. canonicalize semantically relevant structure.
3. serialize canonical form deterministically.
4. hash canonical bytes with configured algorithm family.
5. emit stable graph identity token.

Versioning and compatibility:
- identity algorithm family and canonicalization rules are versioned.
- compatibility windows must preserve comparability guarantees or provide explicit migration mapping.

## Examples
```text
No-op formatting change:
- graph hash before: g_44a...
- graph hash after : g_44a...
Result: same graph identity
```

```text
Dependency change (A no longer depends on B):
- graph hash before: g_44a...
- graph hash after : g_981...
Result: graph identity drift detected
```

## Guarantees
- Equivalent graph semantics produce identical graph identity values.
- Semantic definition changes produce identity drift.
- Identity derivation is deterministic under fixed algorithm/canonicalization version.

## Limitations
- Graph identity does not encode runtime environment state.
- Identity equality does not imply success/failure equivalence for every execution context.
- This document does not define cryptographic library implementation details.

## Related
- `docs/06-specification/01-dag-model.md`
- `docs/06-specification/05-run-identity.md`
- `docs/06-specification/07-replay-semantics.md`
- `docs/05-system-architecture/08-identity-model.md`
