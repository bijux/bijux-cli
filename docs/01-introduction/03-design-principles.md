# Design Principles

## Purpose
Define the durable principles used to evaluate product and documentation decisions.

## Context
These principles apply across runtime behavior, interfaces, and docs.

## Explanation
1. Determinism first
- Stable behavior under equivalent inputs is preferred over convenience shortcuts.

2. Explicit contracts
- Guarantees and boundaries must be named and testable.

3. Inspectability by default
- Operational states and outcomes should be observable without hidden tooling.

4. Replayability as a core capability
- Replay is a normal control loop for validation, not an emergency path.

5. Diff-driven diagnosis
- Differences should be classifiable and attributable.

6. Identity-backed traceability
- Graph, run, and artifact identity should support reasoning across time.

7. Minimal surface area
- Prefer clear, small interfaces over broad ambiguous ones.

8. Honest limitations
- Non-guarantees must be documented where guarantees are documented.

9. Portability with boundaries
- Portability is valuable, but constrained by explicit support contracts.

10. Reader-first documentation
- Docs should optimize for user understanding and action, not internal process narratives.

11. No speculative architecture in reference docs
- Future ideas do not belong in normative user-facing explanations.

12. Operational usefulness over conceptual novelty
- Prefer behavior that is testable, diagnosable, and maintainable.

## Examples
```text
Change evaluation example:
- If a feature increases hidden mutable state, it violates principles 1 and 3.
- If a doc adds broad claims without boundaries, it violates principles 2 and 8.
```

## Guarantees
- The principle set is intentionally stable and reusable.
- Principles can be applied as a review filter for new docs and features.

## Limitations
- Principles are not implementation-level specs.
- Conflicts between principles require maintainer judgment and explicit tradeoff notes.

## Related
- `docs/01-introduction/01-what-is-bijux-dag.md`
- `docs/01-introduction/02-mission.md`
- `docs/08-development/04-contributing.md`
- `docs/06-specification/08-diff-semantics.md`
