# Replay Semantics

Define normative replay behavior, validation outcomes, and bounded divergence handling.

Replay semantics determine whether a historical run can be re-executed and assessed consistently.

## Explanation
Replay definition:
- replay executes a graph using baseline identity context and compares resulting evidence against baseline expectations.

Replay prerequisites:
- accessible baseline run record.
- accessible graph definition or equivalent graph identity material.
- required artifacts/inputs available according to replay mode.

Replay modes:
- strict replay: maximize equivalence requirements for identity and outputs.
- permissive replay: allow configured bounded divergence while preserving explicit classification.

Replay validation outcomes:
- `equivalent`: replay evidence matches baseline contract expectations.
- `drift`: replay completed but one or more identity/output checks diverged.
- `incomplete`: replay could not complete due to missing prerequisites or unsupported features.

Formal replay rules:
- RULE-REPLAY-001: replay MUST emit one explicit classification outcome.
- RULE-REPLAY-002: missing prerequisites MUST classify as `incomplete`, not `equivalent`.
- RULE-REPLAY-003: detected divergence MUST classify as `drift`.
- RULE-REPLAY-004: successful equivalence check MUST classify as `equivalent`.

Determinism rules:
- equivalent semantic inputs under supported environment constraints should converge to equivalent classification.
- environment drift must be surfaced as classified divergence, never silently ignored.

Replay planning rules:
- planner identifies required inputs/artifacts before execution start.
- unresolved prerequisites block strict replay and classify result as incomplete.

Specification consistency rules:
- replay outcome vocabulary must match `docs/06-specification/08-diff-semantics.md`.
- replay guarantees must remain aligned with identity contracts in `04`, `05`, and `06`.

Invalid state definitions:
- INVALID-REPLAY-MISSING-BASELINE: replay requested without baseline run context.
- INVALID-REPLAY-UNCLASSIFIED-RESULT: replay ends without explicit classification.
- INVALID-REPLAY-INCOMPATIBLE-CAPABILITY: required capability absent in selected environment.

Edge cases:
- replay may be `drift` despite equal graph identity when environment inputs differ.
- replay may be `incomplete` for intentionally partial artifact retention policies.

Compatibility notes:
- replay classification semantics are stable; new outcomes require coordinated spec update with diff semantics.

## Examples
```text
Strict replay result:
baseline run: r_100...
replay run  : r_131...
classification: equivalent
```

```text
Replay with missing artifact input:
classification: incomplete
reason: required artifact not found in replay input scope
```

## Guarantees
- Replay provides explicit outcome classification (`equivalent`, `drift`, `incomplete`).
- Missing prerequisites are surfaced as explicit replay failure/incomplete states.
- Replay semantics are identity-aware and auditable.

## Limitations
- Replay does not guarantee identical wall-clock behavior.
- Unsupported backend capabilities can limit strict replay applicability.
- Replay success does not imply complete equivalence of external side effects.

## Related
- `docs/06-specification/04-graph-identity.md`
- `docs/06-specification/05-run-identity.md`
- `docs/06-specification/08-diff-semantics.md`
- `docs/03-user-guide/05-replay.md`

## Invalid and impossible replay states

Additional invalid states:

- INVALID-REPLAY-MISSING-CLASSIFIER-VERSION: result emitted without classification policy reference.
- INVALID-REPLAY-BASELINE-IDENTITY-CONFLICT: baseline run identity and declared graph identity cannot be reconciled.

Impossible-state rule:

- replay MUST NOT emit `equivalent` when prerequisite resolution failed or when any required comparison scope is `unknown`.

## Meaning of replay proof and equivalence

- replay proof: structured evidence package showing how replay classification was produced.
- equivalence (replay): classification that all required replay comparison scopes matched under the declared policy and capability envelope.

Proof is evidence of decision quality, not a claim of universal cross-environment sameness.
