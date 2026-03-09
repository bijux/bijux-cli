# Replay Commands

## Purpose
Document replay command usage for validation and drift detection workflows.

## Context
Replay commands are typically used with run history, inspect, and diff commands.

## Explanation
Replay operations:
- replay a run context
- evaluate replay status for equivalence/mismatch

Common flags:
- `--run-id <id>` baseline selector
- optional DAG selector where supported
- `--output <format>` where supported

Error handling guidance:
- unknown run ID: lookup error
- unsupported replay context: compatibility/runtime error

## Examples
```bash
bijux-dag replay --run-id RUN_20260309_220 --output json
bijux-dag replay --run-id RUN_20260309_220 --dag ./pipelines/main.dag.json --output json
```

```json
{
  "run_id": "RUN_20260309_220",
  "replay_status": "mismatch"
}
```

## Guarantees
- Replay workflow is documented as explicit validation path.
- Integration with inspect and diff is clear.

## Limitations
- Replay equivalence remains bounded by environment/backend constraints.
- Replay algorithm internals are not defined here.

## Related
- `docs/04-cli-reference/06-diff-commands.md`
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/03-user-guide/05-replay.md`
- `docs/06-specification/07-replay-semantics.md`
