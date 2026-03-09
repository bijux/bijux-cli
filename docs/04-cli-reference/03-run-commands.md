# Run Commands

Document run command usage for execution and run-history workflows.

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

Command discovery:
- `bijux-dag run --help`
- `bijux-dag run history --help`

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
- `docs/04-cli-reference/02-dag-commands.md`
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/04-cli-reference/07-replay-commands.md`
- `docs/03-user-guide/04-run-history.md`

## Definitive run command usage model

Use `run` commands as the authoritative path for creating and locating execution evidence:

- create run: execute validated graph and record run identity.
- list history: locate baseline, candidate, and failing runs.
- select run ID: pass into inspect, replay, and diff flows.

## Run history and run inspection examples

```bash
bijux-dag run --dag ./pipelines/main.dag.json
bijux-dag run history --limit 10 --output json
bijux-dag inspect run --run-id RUN_20260309_220 --output json
```

Expected outcome pattern:

```text
- new run record created with stable run_id
- history includes new run in ordered index
- inspect returns status and failure/success evidence for that run_id
```

## Run identity semantics in command workflows

Commands depend on run identity as the stable selector for evidence retrieval:

- history discovers candidate IDs.
- inspect resolves one run ID to detailed evidence.
- replay and diff compare behaviors anchored to specific run IDs.

If run identity changes, treat it as new evidence, not an update-in-place of prior execution history.
