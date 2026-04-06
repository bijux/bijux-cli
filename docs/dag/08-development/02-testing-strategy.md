# Testing Strategy

This document defines what each test lane proves, what it does not prove, and how fixtures are governed to keep evidence trustworthy.

## Lane purpose and proof boundaries

Unit lane proves:
- local logic invariants,
- deterministic helper behavior,
- edge-case handling within one component.

Unit lane does not prove:
- cross-component contract compatibility,
- backend capability behavior.

Integration lane proves:
- component interaction contracts,
- normalized outcome behavior across internal boundaries.

Integration lane does not prove:
- full operator workflow semantics across environments.

End-to-end lane proves:
- user-visible workflows from DAG input to run/artifact/replay/diff outcomes,
- release-critical evidence paths.

End-to-end lane does not prove:
- universal backend equivalence outside tested matrix.

Fast lane (smoke) proves:
- essential health of critical paths on every change.

Fast lane does not replace:
- deep scenario coverage,
- compatibility and degradation checks.

## Good tests and bad tests

Good tests:
- assert contract-level outcomes,
- use deterministic inputs,
- include clear failure reasons tied to guarantees.

Bad tests:
- depend on timing races or mutable external state,
- assert formatting noise instead of semantic behavior,
- duplicate existing coverage without new guarantee.

## Fixture discipline

Fixture rules:
- each fixture maps to a named contract expectation,
- fixtures remain minimal but semantically representative,
- fixture provenance is known and reviewable,
- fixture updates explain which guarantee changed and why.

Treat external fixture imports as trust-boundary crossings and re-verify expected classifications.

## Trust boundaries in testing

Testing results are only as strong as the boundary assumptions:
- environment assumptions must be explicit,
- backend capability assumptions must be declared,
- unknown or incomplete evidence must remain explicit.

## Guarantees

- Lane responsibilities are explicit and non-overlapping.
- Fixture governance is tied to contract truth, not convenience.

## Non-guarantees

- Complete defect absence.
- Cross-environment claims without matching matrix evidence.

## Next reading

- [CI integration](docs/07-operations/01-ci-integration.md)
- [Replay semantics contract](docs/06-specification/07-replay-semantics.md)
- [Diff semantics contract](docs/06-specification/08-diff-semantics.md)
