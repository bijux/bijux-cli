# DAG Model

## Purpose
Define the canonical DAG structure and validation contract used by bijux-dag.

## Context
DAG structure is the root contract for scheduling, execution, replay, and diff behavior.

## Explanation
Canonical DAG entity fields:
- `dag.name`: stable human-readable identifier for the graph definition.
- `dag.nodes`: list of node definitions.
- `dag.edges` or equivalent dependency declarations: directed ordering constraints.
- optional metadata: authoring and annotation fields that do not alter execution semantics unless explicitly declared as semantic inputs.

Node model requirements:
- each node has a unique node identifier within the DAG scope.
- each node declares executable intent (command/action) and explicit inputs.
- node behavior must be representable as deterministic intent under equivalent resolved inputs.

Edge model requirements:
- edges form a directed acyclic graph.
- self-dependency is invalid.
- cycles are invalid.
- missing dependency targets are invalid.

Validation pipeline:
1. parse DAG definition.
2. validate schema shape and required fields.
3. validate node identifier uniqueness.
4. validate dependency target integrity.
5. validate acyclicity.
6. produce canonical representation for scheduling and identity derivation.

Deterministic DAG normalization rules:
- preserve semantic content, discard formatting-only variance.
- normalize field ordering for canonical hash materialization.
- treat explicitly semantic metadata as hash-relevant; non-semantic metadata as hash-irrelevant.

Formal validation rule set:
- RULE-DAG-001: `dag.nodes` MUST be present and non-empty.
- RULE-DAG-002: every node `id` MUST be unique within DAG scope.
- RULE-DAG-003: every `depends_on` reference MUST resolve to an existing node `id`.
- RULE-DAG-004: dependency graph MUST be acyclic.
- RULE-DAG-005: canonicalization MUST preserve semantics and remove non-semantic variance.

Invalid state definitions:
- INVALID-DAG-EMPTY-NODES: node list missing or empty.
- INVALID-DAG-DUPLICATE-NODE-ID: same node identifier appears multiple times.
- INVALID-DAG-UNKNOWN-DEPENDENCY: dependency references unknown node.
- INVALID-DAG-CYCLE-DETECTED: dependency cycle prevents valid topological ordering.

Edge cases:
- isolated source nodes are valid when they represent independent work.
- disconnected DAG components are valid if each component is acyclic and internally consistent.
- declaration ordering differences are valid when canonical semantics are unchanged.

Compatibility notes:
- additional optional metadata fields are allowed if they do not alter existing semantic rules.
- semantic rule changes require explicit versioning and migration guidance.

## Examples
```yaml
dag:
  name: build-and-test
  nodes:
    - id: lint
      run: "cargo clippy --all-targets --all-features"
    - id: test
      run: "cargo test --workspace"
      needs: ["lint"]
```

```text
Validation result:
- valid DAG (acyclic)
- execution order envelope: lint -> test
```

## Guarantees
- DAG validation rejects malformed or cyclic dependency structures.
- Canonicalization rules provide stable scheduling and identity input surfaces.
- Equivalent DAG semantics produce equivalent normalized DAG representations.

## Limitations
- This document does not define backend execution behavior.
- DAG validity does not guarantee runtime success of node commands.
- Schema encoding details may evolve while preserving this semantic contract.

## Related
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/04-graph-identity.md`
- `docs/05-system-architecture/04-scheduler.md`
- `docs/03-user-guide/01-authoring-dags.md`
