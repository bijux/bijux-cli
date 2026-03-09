# Testing Strategy

## Purpose
Define test lanes, fixture governance, and quality gates for maintainable correctness.

## Context
Testing is the executable trust boundary for contracts declared in specification and architecture docs.

## Explanation
Test lane model:
- unit lane: pure logic validation with minimal setup.
- integration lane: cross-component behavior and contract interactions.
- end-to-end lane: full workflow validation (authoring -> run -> artifacts -> inspect/replay/diff).
- smoke lane: fast coverage for release-critical behavior.

Lane quality rules:
- keep unit tests deterministic and isolated.
- integration tests must assert behavior contracts, not implementation trivia.
- end-to-end tests should focus on reader-visible guarantees.
- smoke lane should remain stable and fast for frequent execution.

Fixture governance:
- fixtures must be minimal, versioned, and reproducible.
- fixture changes must include rationale and expected behavior delta.
- avoid oversized fixture trees that obscure intent.

Fixture system guidance:
- store fixtures with clear naming tied to behavior contract under test.
- keep fixture payloads small but semantically representative.
- annotate fixture ownership and intended test lanes.
- version fixture schema assumptions when fixture format evolves.
- delete orphan fixtures that no longer map to active tests.

Failure triage rules:
- classify failures as product regression, test defect, or environment instability.
- flaky tests require immediate stabilization or quarantine with explicit tracking.
- remove outdated tests only when their contract is obsolete and replaced.

## Examples
```bash
cargo test --workspace --locked
```

```text
Fixture review checklist:
- fixture name maps to one behavior contract
- fixture is used by at least one active test
- fixture delta rationale recorded in PR
- fixture remains deterministic across CI/local runs
```

```text
Failure triage example:
- test: replay_equivalence_for_stable_backend
- class: environment instability
- action: isolate external dependency, add deterministic fixture
```

## Guarantees
- Test lane purposes and boundaries are explicit.
- Fixture and triage governance are defined for repeatable quality.
- Release-critical behavior has a dedicated fast verification lane.

## Limitations
- No test strategy can prove complete absence of defects.
- End-to-end coverage depth is bounded by fixture realism and backend availability.
- This document does not prescribe specific CI vendor configuration.

## Related
- `docs/06-specification/07-replay-semantics.md`
- `docs/06-specification/08-diff-semantics.md`
- `docs/07-operations/01-ci-integration.md`
- `docs/08-development/01-repository-structure.md`
