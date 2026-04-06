# Design Principles

These principles are tradeoff rules. Each one exists to force a concrete engineering decision.

## Principle set as enforceable tradeoffs

| Principle | Tradeoff it enforces | Concrete consequence |
| --- | --- | --- |
| Determinism before convenience | reject hidden mutable state shortcuts | classification workflows must stay stable under equivalent inputs |
| Explicit contracts over implied behavior | pay spec/test overhead to avoid ambiguity | invalid states and invariants must be documented, not guessed |
| Inspectability by default | store more structured evidence | run and artifact surfaces must remain queryable after execution |
| Replay as routine control | constrain side-effect-heavy runtime behavior | release confidence can depend on replay classes |
| Diff as decision input | maintain scoped classification logic | graph/run/artifact divergence cannot be collapsed into one result |
| Identity-backed attribution | govern hash/canonicalization policy carefully | graph/run/artifact links must remain machine-resolvable |
| Narrow surface area | trade feature breadth for clarity | commands and contracts stay smaller and more predictable |
| Honest limits near guarantees | accept less marketing-friendly language | every strong claim must carry scope boundary and non-guarantee |
| Portability with capability bounds | reject universal parity claims | backend support classes must be explicit and evidence-based |
| Reader-first reference docs | spend effort on mechanics, not template symmetry | docs must teach action and judgment, not only define terms |
| No speculative architecture in normative docs | separate current truth from ideas | user-facing references must describe implemented behavior only |
| Operational usefulness over novelty | reject elegant but unverifiable designs | CI/tests/docs must prove behavior, not narrate intent |

## How to use these principles in review

Evaluate changes in order:

1. Which principle is improved?
2. Which principle is made worse?
3. Is the tradeoff explicit and documented?
4. Are specs/tests/docs updated to keep the tradeoff honest?

A change that cannot answer these questions is not ready.

## Principle failure examples

- adding backend-specific behavior that bypasses normalized outcomes violates contract explicitness and diff reliability;
- broadening portability language without capability evidence violates honest limits;
- introducing opaque automation that cannot be inspected violates inspectability and replayability.

## Guarantees you can test

- Each principle maps to at least one observable system behavior.
- Each principle includes a real cost, not only a virtue statement.
- Principle checks can be applied during design review and PR review.

## Limits

- Principles do not replace specifications; they constrain design direction.
- Principle conflicts still require maintainer judgment, but tradeoff rationale must be explicit.
- Principles are stable by default and should change only when contract posture changes.

## Next reading

- Product intent that these principles implement: [Mission](../01-introduction/02-mission.md)
- Identity and determinism architecture: [Identity Model](../05-system-architecture/08-identity-model.md)
- Contract-level vocabulary for change classification: [Diff Semantics](../06-specification/08-diff-semantics.md)
- Contribution standards that enforce these principles: [Contributing](../08-development/04-contributing.md)
