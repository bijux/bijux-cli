# Dag Commands

## Purpose
Document DAG command usage for graph validation, inspection, and lifecycle operations.

## Context
DAG commands operate on graph definitions rather than specific runs.

## Explanation
Typical DAG command intents:
- validate graph shape and constraints
- inspect graph metadata and topology
- compare graph definitions via diff workflow integration

Usage guidance:
- validate before running
- keep DAG path explicit and version-controlled

Common options pattern:
- `--dag <path>` to select graph file
- output-format flags when machine parsing is required

## Examples
```bash
bijux-dag dag validate --dag ./pipelines/main.dag.json
bijux-dag dag inspect --dag ./pipelines/main.dag.json
```

```json
{
  "graph_id": "PIPELINE_MAIN",
  "status": "valid"
}
```

## Guarantees
- DAG command family is documented as definition-level surface.
- Validation-first workflow is explicit.

## Limitations
- Exact validation rule internals are specified in specification docs.
- Subcommand availability can vary by release surface.

## Related
- `docs/04-cli-reference/01-cli-overview.md`
- `docs/03-user-guide/01-authoring-dags.md`
- `docs/06-specification/01-dag-model.md`
- `docs/04-cli-reference/06-diff-commands.md`
