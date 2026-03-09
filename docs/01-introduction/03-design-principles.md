# Design Principles

Define the durable principles used to evaluate product and documentation decisions.

These principles apply across runtime behavior, interfaces, and docs.

## Explanation
1. Determinism first
- Stable behavior under equivalent inputs is preferred over convenience shortcuts.
- Why: deterministic systems can be validated and trusted.
- Tradeoff: strict determinism can reduce shortcut flexibility and may require more explicit configuration.
- Example in practice: replay and diff operate on identity-backed evidence, not informal run notes.

2. Explicit contracts
- Guarantees and boundaries must be named and testable.
- Why: unnamed boundaries create hidden assumptions and operational surprises.
- Tradeoff: contract maintenance adds documentation and test discipline cost.
- Example in practice: specification pages define formal outcome vocabularies and invalid states.

3. Inspectability by default
- Operational states and outcomes should be observable without hidden tooling.
- Why: diagnosis quality depends on available evidence.
- Tradeoff: richer evidence surfaces require structured persistence and indexing effort.
- Example in practice: run records and artifact lineage are first-class data surfaces.

4. Replayability as a core capability
- Replay is a normal control loop for validation, not an emergency path.
- Why: reproducibility is strongest when replay is routine.
- Tradeoff: replay support constrains acceptable hidden state and side effects.
- Example in practice: release checks can require replay classification before promotion.

5. Diff-driven diagnosis
- Differences should be classifiable and attributable.
- Why: teams need actionable drift explanation, not generic mismatch signals.
- Tradeoff: classification logic is more complex than binary pass/fail reporting.
- Example in practice: graph/run/artifact scopes classify divergence with reason codes.

6. Identity-backed traceability
- Graph, run, and artifact identity should support reasoning across time.
- Why: traceability enables credible comparisons between historical and candidate behavior.
- Tradeoff: identity derivation must stay versioned and carefully governed.
- Example in practice: artifact lineage joins `graph_id`, `run_id`, `node_id`, and `artifact_id`.

7. Minimal surface area
- Prefer clear, small interfaces over broad ambiguous ones.
- Why: smaller surfaces reduce accidental coupling and cognitive load.
- Tradeoff: some advanced scenarios require explicit extension rather than built-in shortcuts.
- Example in practice: narrow command semantics with explicit capability boundaries.

8. Honest limitations
- Non-guarantees must be documented where guarantees are documented.
- Why: reliability depends on knowing boundaries, not only promises.
- Tradeoff: explicit limitations may look less impressive in shallow comparisons.
- Example in practice: portability docs state support envelope and non-equivalence zones.

9. Portability with boundaries
- Portability is valuable, but constrained by explicit support contracts.
- Why: portable workflows are useful only when equivalence claims are trustworthy.
- Tradeoff: support matrices and validation flows must be maintained.
- Example in practice: backend tiers and capability constraints govern portability assertions.

10. Reader-first documentation
- Docs should optimize for user understanding and action, not internal process narratives.
- Why: docs are operational tools, not archival dumps.
- Tradeoff: writer convenience decreases because content must be curated and pruned.
- Example in practice: user guides focus on executable workflows and troubleshooting paths.

11. No speculative architecture in reference docs
- Future ideas do not belong in normative user-facing explanations.
- Why: speculative text becomes stale and undermines trust.
- Tradeoff: exploratory ideas must live outside normative references.
- Example in practice: architecture pages describe implemented boundaries and explicit non-goals.

12. Operational usefulness over conceptual novelty
- Prefer behavior that is testable, diagnosable, and maintainable.
- Why: production systems fail in operations, not in whiteboard narratives.
- Tradeoff: technically elegant but untestable designs are rejected.
- Example in practice: CI and test lanes prioritize deterministic contract validation.

## Examples
```text
Change evaluation example:
- If a feature increases hidden mutable state, it violates principles 1 and 3.
- If a doc adds broad claims without boundaries, it violates principles 2 and 8.
```

## Guarantees
- The principle set is intentionally stable and reusable.
- Principles can be applied as a review filter for new docs and features.
- Each principle includes rationale, tradeoff, and system-facing example.

## Limitations
- Principles are not implementation-level specs.
- Conflicts between principles require maintainer judgment and explicit tradeoff notes.

## Related
- `docs/01-introduction/01-what-is-bijux-dag.md`
- `docs/01-introduction/02-mission.md`
- `docs/08-development/04-contributing.md`
- `docs/06-specification/08-diff-semantics.md`
