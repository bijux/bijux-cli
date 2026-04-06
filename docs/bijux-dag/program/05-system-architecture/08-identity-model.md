# Identity Model

Identity is the architectural backbone that makes replay and diff trustworthy.

## The three identity surfaces

- graph identity: definition state identity,
- run identity: execution instance identity,
- artifact identity: output unit identity.

These surfaces are complementary. None can substitute for the others.

## Comparison table

| Identity | Primary inputs | Answers |
| --- | --- | --- |
| Graph identity | canonical graph semantics | “What definition state was intended?” |
| Run identity | graph reference + run-attempt uniqueness inputs | “Which execution instance produced this evidence?” |
| Artifact identity | canonical artifact content + identity policy | “Which output unit is this?” |

## Interaction model

1. graph identity scopes intended behavior,
2. run identity records one attempt under that scope,
3. artifact identity links concrete outputs back to that attempt.

Replay/diff consume all three to classify equivalence or drift.

## Common misunderstandings

- equal graph identity does not guarantee equal run outcome,
- equal run identity does not imply every artifact is present or trustworthy,
- equal artifact identity does not prove equal upstream provenance context.

Operational rule: make equivalence claims only when graph, run, and artifact surfaces are coherent for the decision scope.

## Next reading

- Determinism boundaries: [Determinism](../05-system-architecture/09-determinism.md)
- Identity contract details: [Graph Identity](../06-specification/04-graph-identity.md), [Run Identity](../06-specification/05-run-identity.md), [Artifact Identity](../06-specification/06-artifact-identity.md)
