# Authoring Dags

Authoring is not about writing JSON that parses. It is about writing graphs that stay debuggable, replayable, and diffable months later.

## How to think while authoring

Start from data and control dependencies, not from file order. For each node, answer:

- what artifact does this node need,
- what artifact does this node produce,
- which downstream decisions depend on that output.

If you cannot answer those three questions, the node boundary is probably wrong.

## Good habits and bad habits

Good habits:

- single-purpose node intent,
- explicit dependency edges for real prerequisites,
- deterministic output paths,
- artifact boundaries at meaningful transitions.

Bad habits:

- “god nodes” that mix extract/transform/publish in one command,
- hidden dependencies through shared temp paths,
- decorative edges that serialize unrelated work,
- unstable names tied to temporary implementation details.

## Realistic multi-node example

```json
{
  "graph_id": "ORDERS_PIPELINE_V1",
  "nodes": [
    {"id": "extract_orders", "command": "./extract_orders.sh", "depends_on": []},
    {"id": "normalize_orders", "command": "./normalize_orders.sh", "depends_on": ["extract_orders"]},
    {"id": "revenue_report", "command": "./build_revenue_report.sh", "depends_on": ["normalize_orders"]},
    {"id": "quality_checks", "command": "./quality_checks.sh", "depends_on": ["normalize_orders"]}
  ]
}
```

Why this shape works:

- `normalize_orders` is a clear artifact boundary,
- reporting and quality checks fan out from the same normalized dataset,
- failures localize cleanly in inspect and diff.

## Authoring for replayability and diffability

If replay/diff is a requirement, authoring must preserve evidence clarity:

- keep dependency intent explicit so graph diff is meaningful,
- keep output contracts stable so artifact diff is interpretable,
- avoid hidden environment coupling so replay drift is attributable.

## Next reading

- Dependency semantics and runtime order: [Dependencies And Order](../03-user-guide/02-dependencies-and-order.md)
- Artifact lifecycle expectations: [Artifacts](../03-user-guide/03-artifacts.md)
- Formal DAG validity rules: [DAG Model Specification](../06-specification/01-dag-model.md)
