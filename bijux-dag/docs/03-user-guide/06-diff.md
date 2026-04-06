# Diff

Diff answers three operator questions: what changed, where it changed, and whether the change matters.

## Start from the question, then pick surface

- definition changed? use `diff graph`.
- execution behavior changed? use `diff run`.
- delivered outputs changed? use `diff artifact`.

Use all three when release confidence depends on full traceability.

## Full examples by surface

Graph diff:

```bash
bijux-dag diff graph --left ./pipelines/baseline.dag.json --right ./pipelines/candidate.dag.json
```

Run diff:

```bash
bijux-dag diff run --left RUN_20260309_204 --right RUN_20260309_221
```

Artifact diff:

```bash
bijux-dag diff artifact --left ART_orders_v1 --right ART_orders_v2
```

## Semantic versus cosmetic difference

Cosmetic difference example:

- whitespace/comment changes in DAG file,
- graph identity unchanged,
- graph diff should classify equivalent.

Semantic drift example:

- dependency edge changed,
- scheduling eligibility and downstream outcomes changed,
- graph and/or run diff should classify drift.

## Read results without overreacting

Not every drift is a release blocker. Evaluate in order:

1. scope (`graph`, `run`, `artifact`),
2. reason code,
3. contract impact.

Use this sequence to avoid turning every change into incident severity.

## Next reading

- Replay-driven validation context: [Replay](../03-user-guide/05-replay.md)
- Debug workflow using inspect + replay + diff: [Inspect And Debug](../03-user-guide/07-inspect-and-debug.md)
- Formal classification rules: [Diff Semantics Specification](../06-specification/08-diff-semantics.md)
