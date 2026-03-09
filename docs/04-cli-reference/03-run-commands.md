# Run Commands

## Purpose
Document execution and run-history command usage.

## Context
Run commands are the primary operational surface for executing workflows and reviewing run outcomes.

## Explanation
Core run command intents:
- execute DAGs
- inspect run status summaries
- query recent run history

Usage guidance:
- capture `run_id` immediately after execution
- use run history to locate baseline and failing runs

Typical option pattern:
- `--dag <path>` for run creation
- `--run-id <id>` for run-specific follow-up
- `--limit <n>` for history queries

## Examples
```bash
bijux-dag run --dag ./pipelines/main.dag.json
bijux-dag run history --limit 20
```

```json
{
  "run_id": "RUN_20260309_220",
  "status": "completed"
}
```

## Guarantees
- Run command family use is documented as execution-first workflow.
- History usage is integrated as standard operation.

## Limitations
- This document does not specify storage backend implementation details.
- Exact run schema fields are defined in specification docs.

## Related
- `docs/04-cli-reference/01-cli-overview.md`
- `docs/03-user-guide/04-run-history.md`
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/04-cli-reference/07-replay-commands.md`
