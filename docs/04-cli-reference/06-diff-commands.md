# Diff Commands

## Purpose
Document command usage for graph, run, and artifact diff operations.

## Context
Diff commands classify behavioral and structural changes.

## Explanation
Diff command intents:
- `diff graph` for definition changes
- `diff run` for execution outcome changes
- `diff artifact` for output changes

Options and flags pattern:
- `--left <id-or-path>`
- `--right <id-or-path>`
- output-format flags for machine parsing

Error handling guidance:
- incomparable entities should be treated as input error
- missing IDs/paths produce lookup errors

## Examples
```bash
bijux-dag diff graph --left ./pipelines/a.dag.json --right ./pipelines/b.dag.json
bijux-dag diff run --left RUN_20260309_220 --right RUN_20260309_221
bijux-dag diff artifact --left ART_001 --right ART_002
```

```json
{
  "diff_scope": "run",
  "classification": "suspicious_change"
}
```

## Guarantees
- Diff command family usage is explicit across all three scopes.
- Shared option pattern is documented consistently.

## Limitations
- Full classification taxonomy details belong to specification docs.
- This page does not define underlying comparison algorithms.

## Related
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/04-cli-reference/07-replay-commands.md`
- `docs/03-user-guide/06-diff.md`
- `docs/06-specification/08-diff-semantics.md`
