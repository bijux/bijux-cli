# First Dag

## Purpose
Guide a new user from an empty project to a valid minimal DAG definition and first successful run.

## Context
This is the first practical tutorial after installation.

## Explanation
A minimal DAG needs:
- graph identity metadata
- at least one node
- explicit dependency order when multiple nodes exist

Start with two nodes where one depends on the other. This makes ordering explicit and easy to inspect.

Authoring flow:
1. Create DAG file.
2. Define nodes and deterministic command behavior.
3. Define dependency edge.
4. Validate and run.

Dependency walkthrough:
- `build_inputs` has no prerequisites.
- `transform_data` depends on `build_inputs`.
- Scheduler can execute only when dependencies are satisfied.

Node definition walkthrough:
- `id`: stable node identity label.
- `command`: executable behavior.
- `depends_on`: prerequisite node IDs.

## Examples
```json
{
  "graph_id": "EXAMPLE_GRAPH_001",
  "nodes": [
    {
      "id": "build_inputs",
      "command": "echo preparing-inputs > out/input.txt",
      "depends_on": []
    },
    {
      "id": "transform_data",
      "command": "cat out/input.txt > out/result.txt",
      "depends_on": ["build_inputs"]
    }
  ]
}
```

```bash
# Run your first DAG
bijux-dag run --dag ./examples/first.dag.json
```

```text
Expected success indicators:
- run_id emitted
- status reports successful completion
- artifact/output path references are present
```

## Guarantees
- The tutorial DAG is minimal but valid for dependency-order learning.
- The flow shows explicit node and dependency modeling.

## Limitations
- This tutorial omits advanced runtime options and backend variation.
- JSON schema details are covered in specification docs.

## Related
- `docs/02-getting-started/03-running-a-pipeline.md`
- `docs/03-user-guide/01-authoring-dags.md`
- `docs/03-user-guide/02-dependencies-and-order.md`
- `docs/06-specification/01-dag-model.md`
