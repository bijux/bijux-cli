# Dag Commands

Use `dag` commands to prove definition validity before execution. This surface is definition-oriented, not run-history oriented.

## What this command family is for

`dag` commands answer:

- Is this graph valid?
- What is this graph’s canonical identity?
- What changed at definition level?

If a graph fails here, execution commands should not be trusted as diagnostic tools yet.

## Core invocation patterns

```bash
bijux-dag dag --help
bijux-dag dag validate --dag ./pipelines/main.dag.json
bijux-dag dag inspect --dag ./pipelines/main.dag.json --output json
```

If your build exposes a dedicated identity/hash action, use it for gate checks; otherwise read `graph_hash` from inspect JSON output.

## Inputs, outputs, and failure modes

Primary inputs:

- DAG path,
- optional machine-output mode.

Expected outputs:

- validation status,
- graph identity fields (`graph_id`, optional `graph_hash`),
- diagnostics for schema/dependency violations.

Failure classes:

- input failure: unreadable/missing DAG path,
- parse/shape failure: malformed document,
- semantic failure: cycle, unknown dependency target, duplicate node id.

## JSON behavior for automation

Automation should treat `dag` output as gate evidence:

- parse stable keys for status and identity,
- persist validation diagnostics for CI review,
- fail fast on non-zero exit.

Example JSON shape:

```json
{
  "status": "valid",
  "graph_id": "PIPELINE_MAIN",
  "graph_hash": "sha256:5a6f...",
  "node_count": 12,
  "edge_count": 16
}
```

## Next reading

- Run-time execution surface: [Run Commands](../04-cli-reference/03-run-commands.md)
- Definition-level contract semantics: [DAG Model Specification](../06-specification/01-dag-model.md)
