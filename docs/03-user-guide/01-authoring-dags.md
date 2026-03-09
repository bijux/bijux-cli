# Authoring Dags

## Purpose
Define practical authoring rules for building clear, maintainable DAG definitions.

## Context
This document is the primary user guide for writing DAG files beyond the first tutorial.

## Explanation
Authoring goals:
- explicit node intent
- explicit dependencies
- deterministic command behavior

Recommended DAG file structure:
- graph metadata (`graph_id` and optional descriptive fields)
- node list
- per-node command and dependency declarations

Node definition guidance:
- `id` must be stable and descriptive
- `command` should avoid hidden mutable behavior
- `depends_on` should represent real prerequisites only

Execution-order guidance:
- model true constraints, not incidental preference
- avoid over-constraining parallelizable nodes

## Examples
```json
{
  "graph_id": "ETL_GRAPH_001",
  "nodes": [
    {"id": "extract", "command": "./extract.sh", "depends_on": []},
    {"id": "transform", "command": "./transform.sh", "depends_on": ["extract"]},
    {"id": "publish", "command": "./publish.sh", "depends_on": ["transform"]}
  ]
}
```

## Guarantees
- Authoring guidance prioritizes explicit dependency semantics.
- Documented structure aligns with getting-started and specification DAG model docs.

## Limitations
- This guide does not define full schema validation rules.
- Backend-specific command execution constraints are covered elsewhere.

## Related
- `docs/03-user-guide/02-dependencies-and-order.md`
- `docs/02-getting-started/02-first-dag.md`
- `docs/06-specification/01-dag-model.md`
- `docs/06-specification/04-graph-identity.md`
