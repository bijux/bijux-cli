# Contributing

This guide defines the practical path from idea to merged change, with strict expectations on scope, terminology, and evidence.

## Contributor flow

1. define one durable change intent.
2. locate affected contract surface (spec, user guide, operations, development docs, or code).
3. implement the minimal coherent change set.
4. provide evidence (tests, replay/diff outputs, or rationale for doc-only changes).
5. submit review-ready commits with clear Conventional Commit subjects.

## Avoiding scope widening and vocabulary drift

Before opening a PR:
- confirm every edited file is necessary for one intent,
- align terminology with canonical docs,
- avoid introducing synonyms for established terms,
- split unrelated edits into separate commits.

If the change starts touching unrelated guarantees, split and stop extending the PR.

## What maintainers reject immediately

- ambiguous commit intent (`misc`, `cleanup`, vague scope),
- claims of guarantees without boundaries,
- contract-changing code without matching spec updates,
- tests removed without replacement proof,
- documentation that reintroduces template filler over mechanics,
- backend changes that hide capability degradation.

## Choosing the right boundary for a change

Use this boundary selection:
- behavior rule changed -> specification + implementation + tests,
- operator workflow changed -> operations docs + command behavior evidence,
- contributor process changed -> development docs only unless runtime behavior changed,
- wording only -> docs-only commit with no semantic claim changes.

## Checklist: good bijux-dag changes

- one durable intent,
- precise terminology,
- explicit guarantees and non-guarantees,
- evidence aligned with touched surface,
- commit history readable two years later.

## Checklist: bad bijux-dag changes

- mixed intents in one commit,
- hidden semantic change under naming/formatting edits,
- new terms that conflict with canonical vocabulary,
- missing explanation of degraded guarantees,
- unverified boundary crossings.

## Guarantees

- Contribution expectations are concrete and reviewable.
- Scope and terminology discipline are first-class requirements.

## Non-guarantees

- Automatic merge for complete checklists.
- Substitution for technical judgment on complex architecture decisions.

Contributor responsibility: preserve truth in claims, keep scope explicit, and never trade semantic clarity for short-term velocity.

## Next reading

- [Repository structure](01-repository-structure.md)
- [Testing strategy](02-testing-strategy.md)
- [Specification index](../06-specification/01-dag-model.md)
