# First Dag

Create a small but real graph that produces a visible artifact and has meaningful dependency behavior.

## Author the graph

Save this as `./examples/first-orders.dag.json`:

```json
{
  "graph_id": "ORDERS_FIRST_GRAPH",
  "nodes": [
    {
      "id": "extract_orders",
      "command": "printf 'order_id,total\n1,19\n2,42\n' > out/orders_raw.csv",
      "depends_on": []
    },
    {
      "id": "summarize_orders",
      "command": "tail -n +2 out/orders_raw.csv | awk -F, '{sum += $2} END {print sum}' > out/orders_total.txt",
      "depends_on": ["extract_orders"]
    }
  ]
}
```

Inline field meaning:

- `graph_id`: stable identity label for this definition.
- `nodes`: executable units in this graph.
- `id`: unique node key used by dependencies.
- `command`: execution intent for the node.
- `depends_on`: required predecessors.

## Why this graph is valid

- Node IDs are unique.
- `summarize_orders` depends on an existing node (`extract_orders`).
- Dependency topology is acyclic.
- Each node has explicit executable intent.

These map directly to DAG validation rules.

## Run it and verify success

```bash
bijux-dag run --dag ./examples/first-orders.dag.json
```

Expected success signals:

```text
- run_id is returned (example: RUN_20260309_301)
- terminal status is succeeded
- artifact/output references include out/orders_raw.csv and out/orders_total.txt
```

Follow-up checks:

```bash
bijux-dag inspect run --run-id RUN_20260309_301
bijux-dag inspect artifact --run-id RUN_20260309_301
```

Interpretation:

- inspect run should show both nodes as succeeded,
- inspect artifact should show lineage from each output to producing node.

## Next reading

- Full execution and evidence walkthrough: [Running A Pipeline](../02-getting-started/03-running-a-pipeline.md)
- How run records are structured and interpreted: [Understanding Runs](../02-getting-started/04-understanding-runs.md)
