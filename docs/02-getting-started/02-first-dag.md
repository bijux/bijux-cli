# First Dag

## Purpose
Guide a new user from empty workspace to a valid DAG with explicit execution order.

## Context
This is the first hands-on step after installation.

## Explanation
Use a two-node DAG to make ordering explicit and easy to debug.

Beginner walkthrough:
1. Create a DAG file.
2. Define two nodes.
3. Make node B depend on node A.
4. Run the DAG.
5. Inspect run output.

Beginner mental model:
- A graph is a plan.
- A run is one execution of that plan.
- A dependency edge means "must run after".

Quick start summary:
- Author DAG
- Run DAG
- Inspect run
- Replay if needed
- Diff when behavior changes

## Examples
```json
{
  "graph_id": "EXAMPLE_GRAPH_001",
  "nodes": [
    {
      "id": "prepare",
      "command": "echo ok > out/input.txt",
      "depends_on": []
    },
    {
      "id": "transform",
      "command": "cat out/input.txt > out/result.txt",
      "depends_on": ["prepare"]
    }
  ]
}
```

```bash
bijux-dag run --dag ./examples/first.dag.json
bijux-dag inspect run --run-id RUN_20260309_001
```

## Guarantees
- This tutorial uses explicit dependency ordering.
- Example shape is intentionally minimal and readable.

## Limitations
- Advanced backend/runtime options are out of scope.
- Schema-level constraints are documented in specification docs.

## Related
- `docs/02-getting-started/03-running-a-pipeline.md`
- `docs/02-getting-started/04-understanding-runs.md`
- `docs/03-user-guide/01-authoring-dags.md`
- `docs/06-specification/01-dag-model.md`
