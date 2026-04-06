# Diff Commands

Use `diff` commands to classify change across graph, run, and artifact surfaces.

## Surface semantics

- `diff graph`: definition semantics changed or equivalent.
- `diff run`: execution outcomes changed or equivalent.
- `diff artifact`: output identities/lineage changed or equivalent.

Choose surface by question; do not force one surface to answer all questions.

## Core invocation patterns

```bash
bijux-dag diff --help
bijux-dag diff graph --left ./pipelines/a.dag.json --right ./pipelines/b.dag.json --output json
bijux-dag diff run --left RUN_20260309_220 --right RUN_20260309_221 --output json
bijux-dag diff artifact --left ART_001 --right ART_002 --output json
```

## Classification interpretation

- `equivalent`: no contract-relevant divergence on requested surface.
- `drift`: divergence detected and attributed.
- `unknown`/incomplete-style states: required evidence missing or incomparable.

Operator rule: unresolved states require more evidence, not forced acceptance.

## Example outcomes

Equivalent run case:

```text
diff_scope: run
classification: equivalent
reason_code: NONE
```

Drift artifact case:

```text
diff_scope: artifact
classification: drift
reason_code: ARTIFACT_HASH_MISMATCH
```

## Next reading

- Replay pairing and downgrade handling: [Replay Commands](../04-cli-reference/07-replay-commands.md)
- Formal diff contract: [Diff Semantics Specification](../06-specification/08-diff-semantics.md)
