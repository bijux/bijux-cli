# Dag Commands

## Purpose
Document DAG command usage for graph validation, inspection, and definition-level operations.

## Context
DAG commands act on graph definitions, not on specific historical runs.

## Explanation
Common DAG operations:
- validate graph shape and constraints
- inspect graph metadata and topology

Common flags:
- `--dag <path>` graph file selector
- `--output <format>` machine-readable output where supported

Error handling guidance:
- missing DAG path: input error
- invalid graph structure: validation error

## Examples
```bash
bijux-dag dag validate --dag ./pipelines/main.dag.json
bijux-dag dag inspect --dag ./pipelines/main.dag.json --output json
```

```json
{
  "graph_id": "PIPELINE_MAIN",
  "status": "valid"
}
```

## Guarantees
- DAG command behavior is documented as definition-first.
- Validation-first usage is explicit.

## Limitations
- Full validation rule taxonomy belongs to specification docs.
- Option names can evolve across releases.

## Related
- `docs/04-cli-reference/01-cli-overview.md`
- `docs/03-user-guide/01-authoring-dags.md`
- `docs/06-specification/01-dag-model.md`
- `docs/04-cli-reference/06-diff-commands.md`
