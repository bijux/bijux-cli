# Replay Commands

## Purpose
Document replay command usage for validation and drift detection workflows.

## Context
Replay commands are used with run history and diff commands to verify behavioral stability.

## Explanation
Replay command intents:
- rerun/validate prior run context
- compare expected and observed behavior over time

Options and flags pattern:
- `--run-id <id>` baseline selector
- optional DAG/path selectors when surface supports explicit override

Output usage:
- inspect replay status first
- route mismatch findings into diff flow

## Examples
```bash
bijux-dag replay --run-id RUN_20260309_220
bijux-dag replay --run-id RUN_20260309_220 --dag ./pipelines/main.dag.json
```

```json
{
  "run_id": "RUN_20260309_220",
  "replay_status": "mismatch"
}
```

## Guarantees
- Replay command workflow is documented with explicit follow-up behavior.
- Integration with diff/inspect is clear.

## Limitations
- Replay equivalence remains bounded by environment and backend support.
- Algorithmic replay internals are specified elsewhere.

## Related
- `docs/04-cli-reference/06-diff-commands.md`
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/03-user-guide/05-replay.md`
- `docs/06-specification/07-replay-semantics.md`
