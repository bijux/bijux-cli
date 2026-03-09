# Dag Commands

## Purpose
Document DAG command usage for graph validation, inspection, and definition-level operations.

## Context
DAG commands act on graph definitions, not on specific historical runs.

## Explanation
Common DAG operations:
- validate graph shape and constraints.
- inspect graph metadata and topology.
- provide definition-level diagnostics before execution.

Common flags:
- `--dag <path>` graph file selector
- `--output <format>` machine-readable output where supported

Command lifecycle role:
- DAG commands are pre-run controls.
- use `dag validate` before every new or changed pipeline execution.
- use `dag inspect` when debugging definition-level drift.

Error handling guidance:
- missing DAG path: input error
- invalid graph structure: validation error
- non-readable DAG file path: filesystem input error

## Examples
```bash
# Validate definition before execution
bijux-dag dag validate --dag ./pipelines/main.dag.json

# Inspect topology metadata for review
bijux-dag dag inspect --dag ./pipelines/main.dag.json --output json
```

```json
{
  "graph_id": "PIPELINE_MAIN",
  "status": "valid",
  "node_count": 12,
  "edge_count": 16
}
```

```text
Failure example:
command: bijux-dag dag validate --dag ./pipelines/invalid-cycle.dag.json
result: non-zero exit
reason: cycle detected in dependency graph
```

## Guarantees
- DAG command behavior is documented as definition-first.
- Validation-first usage is explicit.
- Examples cover normal and failing definition workflows.

## Limitations
- Full validation rule taxonomy belongs to specification docs.
- Option names can evolve across releases.

## Related
- `docs/04-cli-reference/01-cli-overview.md`
- `docs/03-user-guide/01-authoring-dags.md`
- `docs/06-specification/01-dag-model.md`
- `docs/04-cli-reference/06-diff-commands.md`
