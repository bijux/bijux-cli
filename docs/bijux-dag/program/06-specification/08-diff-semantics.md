# Diff Semantics Specification

Diff semantics define how differences are classified across graph, run, and artifact surfaces. The contract exists so operators can answer three concrete questions: what changed, where it changed, and whether the change is contract-relevant.

## Contract surface

This specification defines:
- diff surfaces,
- classification vocabulary,
- rules for equivalent vs drift vs unknown,
- semantic-drift versus cosmetic-difference interpretation.

This specification does not define remediation policy or UI presentation.

## Diff surfaces

Supported comparison surfaces:
- `graph`: canonical DAG semantics,
- `run`: run-attempt evidence and outcomes,
- `artifact`: canonical artifact payload identity.

Composite diff results MAY contain multiple surfaces. Each surface MUST be classified independently.

## Classification vocabulary

- `equivalent`: no contract-relevant divergence for the requested surface under declared policy.
- `drift`: contract-relevant divergence detected for the requested surface.
- `unknown`: required evidence or capability missing for the requested surface.

## Normative rules

- `RULE-DIFF-001`: each diff result MUST include surface and classification.
- `RULE-DIFF-002`: unresolved required evidence MUST classify as `unknown`.
- `RULE-DIFF-003`: detected contract-relevant divergence MUST classify as `drift`.
- `RULE-DIFF-004`: `equivalent` is valid only when requested surface has no contract-relevant divergence.
- `RULE-DIFF-005`: reason codes MUST be emitted for `drift` and `unknown`.

## Semantic drift versus cosmetic difference

Cosmetic difference means representation changed but canonical semantics did not change.

Semantic drift means canonical semantics changed in a way relevant to the requested surface.

Example cosmetic difference:

```text
graph file whitespace/comments changed
canonical graph unchanged
classification: graph = equivalent
```

Example semantic drift:

```text
dependency edge removed from transform -> validate
execution frontier changes
classification: graph = drift
```

## Exact meaning of equivalent

`equivalent` means equivalence for the requested surface, under declared policy and available evidence.

`equivalent` does not mean:
- equal wall-clock behavior,
- equal resource profile,
- equal external side effects outside requested surface.

## Invalid states

- `INVALID-DIFF-MISSING-SURFACE`: surface not declared.
- `INVALID-DIFF-MISSING-CLASSIFICATION`: classification not declared.
- `INVALID-DIFF-ILLEGAL-COERCION`: unresolved scope coerced from `unknown` to `equivalent` or `drift` without evidence.

Implementations MUST reject invalid diff results.

## Worked examples

Composite diff with mixed classifications.

```text
graph: equivalent
run: drift (reason_code: NODE_TEST_FAILED)
artifact: unknown (reason_code: ARTIFACT_NOT_RETAINED)
```

Artifact drift.

```text
baseline artifact: a_712...
candidate artifact: a_7af...
classification: drift
reason_code: ARTIFACT_HASH_MISMATCH
```

## Guarantees

- Diff output is explicit and surface-scoped.
- Unknown state is represented explicitly instead of hidden.
- Diff semantics align with replay semantics vocabulary.

## Non-guarantees

- Diff does not prescribe automatic rollback or remediation.
- Diff does not guarantee complete conclusions when evidence is missing.
- Diff does not replace policy decisions about acceptable drift.

## Next reading

- [Replay semantics contract](07-replay-semantics.md)
- [Artifact identity contract](06-artifact-identity.md)
- [User guide: diff interpretation](../03-user-guide/06-diff.md)
