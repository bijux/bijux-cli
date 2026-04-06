# Determinism

Bijux-dag claims bounded determinism, not universal sameness.

## Determinism categories

- deterministic identity: same canonical inputs produce same identity values.
- deterministic planning: same dependency state produces same readiness semantics.
- deterministic execution: equivalent inputs within capability envelope produce equivalent classified outcomes.
- deterministic reporting: classification vocabulary and reason-code semantics stay stable for comparable evidence.

## What bijux-dag claims

Claims:

- identity derivation is policy-governed and reproducible,
- scheduler eligibility is dependency-correct and stable in meaning,
- replay/diff classification semantics are explicit and contract-bound.

Non-claims:

- identical wall-clock interleaving for parallel nodes,
- identical performance across backend families,
- equivalence beyond declared capability and environment envelope.

## Backend and environment effects

Determinism can degrade when:

- adapter capabilities differ,
- toolchain/runtime environment drifts,
- required evidence surfaces are missing.

In those cases, system should classify bounded or incomplete states explicitly rather than forcing equivalence.

## Next reading

- Capability envelope and portability impact: [Portability](../05-system-architecture/10-portability.md)
- Contract-level semantics: [Replay Semantics Specification](../06-specification/07-replay-semantics.md), [Diff Semantics Specification](../06-specification/08-diff-semantics.md)
