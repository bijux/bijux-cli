# Run Commands

## Purpose
Document run command usage for execution and run-history workflows.

## Context
Run commands are the primary operator surface for starting workflows and reviewing results.

## Explanation
Common run operations:
- execute a DAG.
- view run history.
- retrieve run-scoped operational status.

Common flags:
- `--dag <path>` for run creation
- `--run-id <id>` for targeted run follow-up
- `--limit <n>` for history windowing
- `--output <format>` where supported

Command lifecycle role:
- run execution is the source of run IDs used by inspect/replay/diff.
- run history is the indexing surface for selecting baselines and failing candidates.

Error handling guidance:
- invalid DAG: validation error
- unknown run ID: lookup error
- runtime node failure: run completes with failed status and non-zero exit where applicable

## Examples
```bash
# Start a run
bijux-dag run --dag ./pipelines/main.dag.json

# Query recent history
bijux-dag run history --limit 20 --output json
```

```json
{
  "run_id": "RUN_20260309_220",
  "status": "completed",
  "graph_id": "PIPELINE_MAIN"
}
```

```text
Run lifecycle command flow:
1) run --dag ...
2) run history --limit ...
3) inspect run --run-id ...
```

## Guarantees
- Run command usage is documented as execution-plus-history flow.
- Command examples align with user-guide run workflows.
- Flags and output examples support both human and automation usage.

## Limitations
- Storage engine internals are outside CLI reference scope.
- Detailed run schema contract is defined in specification docs.

## Related
- `docs/04-cli-reference/01-cli-overview.md`
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/04-cli-reference/07-replay-commands.md`
- `docs/03-user-guide/04-run-history.md`
