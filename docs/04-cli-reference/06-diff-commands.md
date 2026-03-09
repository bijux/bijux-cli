# Diff Commands

Document command usage for graph, run, and artifact diff operations.

Diff commands are used to classify change and isolate causes of divergence.

## Explanation
Diff operations:
- `diff graph`
- `diff run`
- `diff artifact`

Common flags:
- `--left <id-or-path>`
- `--right <id-or-path>`
- `--output <format>` where supported

Command lifecycle role:
- `diff graph` for definition drift.
- `diff run` for behavioral drift.
- `diff artifact` for output drift.
- run scopes in that order when triaging unknown regressions.

Command discovery:
- `bijux-dag diff --help`
- `bijux-dag diff graph --help`
- `bijux-dag diff run --help`
- `bijux-dag diff artifact --help`

Error handling guidance:
- incomparable entities: input/compatibility error
- missing IDs or paths: lookup/input error
- unsupported comparison scope: compatibility/runtime error

## Examples
```bash
bijux-dag diff graph --left ./pipelines/a.dag.json --right ./pipelines/b.dag.json --output json
bijux-dag diff run --left RUN_20260309_220 --right RUN_20260309_221 --output json
bijux-dag diff artifact --left ART_001 --right ART_002 --output json
```

```json
{
  "diff_scope": "run",
  "classification": "suspicious_change",
  "reason_code": "NODE_EXIT_NONZERO"
}
```

```text
Command discovery pattern:
bijux-dag diff --help
bijux-dag diff run --help
```

## Guarantees
- Diff usage is documented across all three supported scopes.
- Option patterns are consistent with other command-family docs.
- Examples include classification output fields useful for automation.

## Limitations
- Detailed classification semantics are owned by specification docs.
- This page does not define comparison engine internals.

## Related
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/04-cli-reference/07-replay-commands.md`
- `docs/03-user-guide/06-diff.md`
- `docs/06-specification/08-diff-semantics.md`
