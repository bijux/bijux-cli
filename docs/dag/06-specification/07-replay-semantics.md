# Replay Semantics Specification

Replay semantics define how a baseline run is re-executed and classified. The contract exists so replay results are interpretable, comparable, and auditable instead of ad hoc judgments.

## Contract surface

This specification defines:
- replay prerequisites,
- replay outcome classes,
- invalid and impossible states,
- meaning of replay proof and equivalence.

This specification does not define CLI formatting or scheduling algorithm internals.

## Replay prerequisites

A replay request MUST provide:
- baseline run reference,
- graph definition or graph identity material required by mode,
- required comparison-scope evidence,
- selected capability envelope (backend/environment constraints).

Missing prerequisite material MUST prevent `equivalent` classification.

## Outcome classes

Replay classification vocabulary is fixed:
- `equivalent`: all required replay comparison scopes matched under declared policy and capability envelope.
- `drift`: replay completed and at least one required comparison scope diverged.
- `incomplete`: replay could not classify required scopes because prerequisites or capabilities were missing.

## Normative rules

- `RULE-REPLAY-001`: every replay result MUST emit exactly one classification.
- `RULE-REPLAY-002`: unmet prerequisites MUST classify as `incomplete`.
- `RULE-REPLAY-003`: detected divergence in any required scope MUST classify as `drift`.
- `RULE-REPLAY-004`: `equivalent` is valid only when all required scopes are resolved and non-divergent.
- `RULE-REPLAY-005`: replay result MUST include policy/capability references used for classification.

## Invalid and impossible states

Invalid states:
- `INVALID-REPLAY-MISSING-BASELINE`: baseline run not specified or unresolved.
- `INVALID-REPLAY-NO-CLASSIFICATION`: replay finished without classification.
- `INVALID-REPLAY-MISSING-POLICY`: classification emitted without policy reference.
- `INVALID-REPLAY-CAPABILITY-CONFLICT`: required scope cannot be evaluated in declared capability envelope.

Impossible-state rules:
- replay MUST NOT emit `equivalent` when any required scope is unresolved.
- replay MUST NOT emit `equivalent` when any required scope is divergent.

## Replay proof and equivalence meaning

Replay proof is the evidence package that documents:
- baseline reference,
- candidate run reference,
- evaluated scopes,
- classification and reason codes,
- policy and capability envelope.

Replay proof demonstrates how classification was decided. It is not a claim of universal sameness across every environment.

## Worked examples

Equivalent replay.

```text
baseline: r_100...
candidate: r_131...
required scopes: graph, run, artifact
classification: equivalent
```

Incomplete replay due to missing artifact.

```text
baseline: r_100...
candidate: r_132...
required scope artifact: unavailable
classification: incomplete
reason_code: REPLAY_MISSING_ARTIFACT_SCOPE
```

## Guarantees

- Replay returns explicit, bounded classification.
- Replay never silently upgrades unresolved comparison to `equivalent`.
- Replay proof captures the basis for classification.

## Non-guarantees

- Replay equivalence does not guarantee equal timing or resource profile.
- Replay equivalence does not guarantee equivalence for scopes not requested.
- Replay may be incomplete under bounded backend capabilities.

## Next reading

- [Graph identity contract](docs/06-specification/04-graph-identity.md)
- [Run identity contract](docs/06-specification/05-run-identity.md)
- [Diff semantics contract](docs/06-specification/08-diff-semantics.md)
