# Diff Semantics

Define normative diff classification across graph, run, and artifact domains.

Diff semantics is the comparison contract used by validation, release decisions, and troubleshooting.

## Explanation
Diff scope levels:
- graph diff: compares canonical DAG semantics via graph identity/material.
- run diff: compares run-level outcomes, statuses, and execution evidence.
- artifact diff: compares artifact identity and content equivalence.

Classification vocabulary:
- `equivalent`: no contract-relevant divergence detected.
- `drift`: divergence detected and attributable.
- `unknown`: comparison could not be completed with available evidence.

Formal diff rules:
- RULE-DIFF-001: diff output MUST include scope and classification.
- RULE-DIFF-002: unresolved comparisons MUST classify as `unknown`.
- RULE-DIFF-003: attributable divergence MUST classify as `drift`.
- RULE-DIFF-004: equivalent classification requires absence of contract-relevant divergence for that scope.

Comparison rules:
- diff must operate on canonicalized data where available.
- classification must include machine-readable reason codes.
- unknown states must not be silently downgraded to equivalent.

Determinism compatibility:
- stable identity inputs should yield stable diff outcomes.
- non-deterministic or incomplete inputs must be classified explicitly.

Ambiguity control:
- every diff result must include scope (`graph|run|artifact`) and classification.
- mixed outcomes across scopes must be represented as a structured composite result.

Specification audit and maintenance rules:
- diff classification terms are reserved and cannot be redefined ad hoc.
- any new classification term requires coordinated updates in replay semantics and user guidance.
- wording is intentionally strict to prevent ambiguous interpretation across tooling.

Invalid state definitions:
- INVALID-DIFF-MISSING-SCOPE: scope omitted from diff result.
- INVALID-DIFF-MISSING-CLASSIFICATION: result emitted without classification.
- INVALID-DIFF-UNSUPPORTED-COERCION: unknown state coerced into equivalent or drift without evidence.

Edge cases:
- graph equivalent with run drift is valid and common under environment/input differences.
- artifact comparison can be unknown when one side is missing retained artifact evidence.

Compatibility notes:
- reason-code catalogs may expand, but core classification vocabulary (`equivalent`, `drift`, `unknown`) remains stable.

## Examples
```text
Graph equivalent, run drift:
graph: equivalent
run  : drift (reason: node test failed in candidate run)
artifact: unknown (node output missing)
```

```text
Artifact diff record:
artifact_id_baseline: a_712...
artifact_id_candidate: a_7af...
classification: drift
reason_code: ARTIFACT_HASH_MISMATCH
```

## Guarantees
- Diff output is explicitly classified and scope-aware.
- Unknown comparison states are represented explicitly.
- Diff semantics is aligned with replay and identity contracts.

## Limitations
- Diff classification does not prescribe automatic remediation.
- Some domains may remain unknown when evidence is missing or unsupported.
- This specification does not define UI/CLI rendering format details.

## Related
- `docs/06-specification/07-replay-semantics.md`
- `docs/06-specification/04-graph-identity.md`
- `docs/06-specification/06-artifact-identity.md`
- `docs/03-user-guide/06-diff.md`
