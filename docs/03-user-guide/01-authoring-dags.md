# Authoring Dags

## Purpose
Define practical authoring rules for clear, maintainable DAG definitions.

## Context
This is the canonical user-guide entry for writing DAG files beyond first-run tutorials.

## Explanation
Authoring principles:
- model true dependencies only
- keep node scope single-purpose
- prefer deterministic commands over environment-dependent behavior

Recommended file shape:
- graph identity metadata
- node list with stable IDs
- explicit dependency declarations

Node structure guidance:
- `id`: unique, stable, human-readable node identifier.
- `command`: explicit executable intent; avoid hidden side effects where possible.
- `depends_on`: explicit prerequisites; empty list means source node.
- optional metadata: owner/category/labels when they help operations, but keep semantic meaning clear.

Dependency graph formation model:
- start from true data/control prerequisites.
- create one edge for each required prerequisite relation.
- avoid inferred dependencies hidden in scripts or implicit filesystem state.
- validate that graph has at least one source node and no cycles.

Authoring quality checklist:
1. every node ID is unique and descriptive
2. every dependency target exists
3. no hidden dependency in shell scripts
4. outputs are explicit and inspectable
5. graph is acyclic and schedulable
6. each node command has a clear success/failure signal

Concept boundaries:
- DAG defines execution plan
- run executes that plan
- artifact stores produced outputs

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

```json
{
  "graph_id": "INVALID_CYCLE_EXAMPLE",
  "nodes": [
    {"id": "a", "command": "echo a", "depends_on": ["c"]},
    {"id": "b", "command": "echo b", "depends_on": ["a"]},
    {"id": "c", "command": "echo c", "depends_on": ["b"]}
  ]
}
```

```text
Invalid DAG reason:
- cycle detected (a -> c -> b -> a)
- graph cannot be scheduled
```

```text
Authoring review example:
- if transform depends on extract output, dependency must be explicit
- if publish can run without transform, remove that edge
```

```text
Graph validation expectation:
- unique node IDs: pass
- dependency target existence: pass/fail
- acyclicity: pass/fail
- scheduler readiness: pass/fail
```

## Guarantees
- Guidance is aligned with dependency semantics and scheduler behavior described elsewhere.
- This document is the single user-guide source for DAG authoring baseline.
- Includes explicit valid and invalid authoring patterns.

## Limitations
- Does not define full schema validation internals.
- Does not cover backend-specific command execution constraints.

## Related
- `docs/03-user-guide/02-dependencies-and-order.md`
- `docs/03-user-guide/03-artifacts.md`
- `docs/06-specification/01-dag-model.md`
- `docs/06-specification/04-graph-identity.md`
