# DAG Model

Normative contract for graph definition validity and canonical semantics.

## Terms

- DAG: directed acyclic graph of nodes and dependency edges.
- Node: executable definition unit identified by unique `id`.
- Dependency edge: prerequisite relation constraining execution eligibility.

## Required fields

- `graph_id` (or equivalent definition identifier)
- `nodes` (non-empty list)
- per-node `id`
- per-node executable intent field (command/action)
- dependency declarations (`depends_on` or equivalent)

## Validation rules

- RULE-DAG-001: `nodes` MUST be present and non-empty.
- RULE-DAG-002: node IDs MUST be unique within graph scope.
- RULE-DAG-003: every dependency reference MUST resolve to an existing node ID.
- RULE-DAG-004: dependency graph MUST be acyclic.
- RULE-DAG-005: canonical semantic form MUST be derivable for identity computation.

## Invariants

- dependency relation is directed,
- no self-dependency,
- topological ordering exists for valid graph,
- canonicalization preserves semantic meaning while removing non-semantic variance.

## Invalid states

- INVALID-DAG-EMPTY-NODES
- INVALID-DAG-DUPLICATE-NODE-ID
- INVALID-DAG-UNKNOWN-DEPENDENCY
- INVALID-DAG-CYCLE-DETECTED
- INVALID-DAG-NONCANONICAL-SEMANTIC-RESOLUTION

## Graph-identity relevant fields

Identity-relevant:

- node execution semantics,
- dependency topology,
- semantic config declared identity-relevant by policy.

Identity-irrelevant:

- formatting-only differences,
- comments/whitespace,
- explicitly non-semantic annotations.

## Examples

Valid normalized example:

```json
{
  "graph_id": "BUILD_TEST",
  "nodes": [
    {"id": "lint", "command": "cargo clippy", "depends_on": []},
    {"id": "test", "command": "cargo test", "depends_on": ["lint"]}
  ]
}
```

Invalid example:

```json
{
  "graph_id": "INVALID",
  "nodes": [
    {"id": "train", "command": "./train.sh", "depends_on": ["prepare_data"]}
  ]
}
```

Expected invalid reason: `INVALID-DAG-UNKNOWN-DEPENDENCY`.

## Next reading

- Graph identity derivation: [Graph Identity](../06-specification/04-graph-identity.md)
- Execution semantics over this model: [Run Model](../06-specification/02-run-model.md)
