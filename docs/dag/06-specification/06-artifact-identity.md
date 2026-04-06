# Artifact Identity Specification

Artifact identity is the identifier of canonical artifact content under a declared policy. It exists so systems can compare output equivalence while keeping provenance and lineage as separate evidence.

## Contract surface

This specification defines:
- artifact identity inputs and exclusions,
- provenance and lineage interaction,
- invalid and incomplete comparison states,
- cross-policy compatibility behavior.

This specification does not define transport format details or storage backend design.

## Normative requirements

### Identity inputs

Artifact identity MUST be derived from:
- canonical artifact payload representation,
- artifact-kind canonicalization policy,
- artifact identity policy version.

Artifact identity MUST NOT be derived from:
- storage path unless explicitly declared semantic,
- display metadata,
- transport-container metadata outside canonical payload.

### Core rules

- `RULE-AID-001`: equal canonical payload under same policy MUST yield equal artifact identity.
- `RULE-AID-002`: canonical payload drift MUST yield different artifact identity.
- `RULE-AID-003`: artifact records MUST keep lineage links to producing run and node.
- `RULE-AID-004`: cross-policy comparisons MUST be explicit; incompatible policies cannot be silently treated as equivalent.

## Identity and lineage interaction

Artifact identity answers content equivalence. Lineage answers production context.

Equal identity with different lineage is valid and expected in replay/import scenarios.

## Invalid and incomplete states

- `INVALID-AID-MISSING-CANONICAL-PAYLOAD`: canonical payload unavailable for identity derivation.
- `INVALID-AID-UNKNOWN-POLICY`: policy missing or unsupported.
- `INVALID-AID-MISSING-LINEAGE`: record lacks required producing run/node linkage.
- `INCOMPLETE-AID-COMPARISON`: one side missing required payload or policy to classify equivalence.

Implementations MUST reject invalid records and MUST classify incomplete comparisons explicitly.

## Worked examples

Example: identical payload, different provenance.

```text
artifact A id: a_2f67...
artifact B id: a_2f67...
A lineage: run r_101 / node transform
B lineage: run r_188 / node transform
Result: content-equivalent artifacts with different lineage significance
```

Example: payload drift.

```text
baseline artifact id : a_2f67...
candidate artifact id: a_8ad1...
Result: artifact drift
```

## Guarantees

- Artifact identity is deterministic for canonical payload under fixed policy.
- Identity drift is explicit for canonical payload drift.
- Lineage linkage is preserved as separate evidence.

## Non-guarantees

- Equal artifact identity does not prove equal external side effects.
- Equal artifact identity does not imply same producer run.
- Artifact identity alone is insufficient for full incident attribution.

## Next reading

- [Artifact model contract](03-artifact-model.md)
- [Run identity contract](05-run-identity.md)
- [Diff semantics contract](08-diff-semantics.md)
