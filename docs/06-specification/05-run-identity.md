# Run Identity Specification

Run identity is the identifier of one concrete execution attempt. It exists so history, inspect, replay, and artifact lineage can refer to a specific attempt even when multiple attempts use the same graph identity.

## Contract surface

This specification defines:
- run identity inputs and exclusions,
- uniqueness and attribution rules,
- relationship to graph identity and ancestry,
- invalid states.

This specification does not define storage key naming or UI formatting.

## Normative requirements

### Identity inputs

Run identity MUST be derived from:
- graph identity reference,
- run-attempt uniqueness input inside namespace,
- run identity policy version.

Run identity MAY include additional declared semantic execution selectors if policy marks them identity-relevant.

Run identity MUST NOT be derived from:
- display labels,
- post-hoc analytics,
- non-semantic annotations.

### Core rules

- `RULE-RID-001`: each run attempt MUST have exactly one run identity.
- `RULE-RID-002`: distinct run attempts MUST NOT share run identity in one namespace.
- `RULE-RID-003`: identity derivation MUST be deterministic for the declared input tuple.
- `RULE-RID-004`: run identity MUST remain linkable to graph identity and run ancestry metadata.

## What changes run identity

Changes that MUST change run identity:
- new execution attempt,
- change in namespace or uniqueness input,
- incompatible run identity policy version change.

Changes that MUST NOT change run identity:
- rendering format changes,
- added derived summary fields,
- non-semantic annotation edits.

## Relationship to graph identity and ancestry

- Many runs MAY share one graph identity.
- Run identity identifies attempt-level evidence.
- Ancestry fields classify relationship classes such as original, replayed, and imported.
- Ancestry links MUST NOT collapse distinct run identities into one logical attempt.

## Invalid states

- `INVALID-RID-DUPLICATE`: two distinct attempts share one run identity.
- `INVALID-RID-MISSING-GRAPH-LINK`: run identity exists without graph identity link.
- `INVALID-RID-NONDETERMINISTIC-DERIVATION`: same declared inputs produce different run identities.

Implementations MUST reject invalid run identity records.

## Worked examples

Example: same graph, new attempt.

```text
graph_id: g_44a...
run #1: r_100...
run #2: r_101...
Result: valid (same graph identity, distinct run identities)
```

Example: metadata-only change.

```text
Before: run r_101... label="nightly"
After : run r_101... label="nightly-main"
Result: run identity unchanged
```

## Guarantees

- Run identity is unique per attempt within namespace.
- Run identity gives stable attempt-level reference for evidence and lineage.
- Distinct attempts remain distinguishable even with identical graph identity.

## Non-guarantees

- Run identity does not imply artifact content equivalence.
- Run identity does not imply equal wall-clock behavior.
- Run identity equality across different namespaces is not guaranteed.

## Next reading

- [Run model contract](docs/06-specification/02-run-model.md)
- [Graph identity contract](docs/06-specification/04-graph-identity.md)
- [Artifact identity contract](docs/06-specification/06-artifact-identity.md)
