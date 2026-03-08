# Node Identity Contract

## Scope

Defines stable node identity semantics for graph authoring, planning, execution traces,
and artifact lineage.

## Canonical node identifier

- Canonical field: `node.id` (string) in DAG documents.
- `node.id` must be unique within a graph.
- `node.id` is stable across planning and runtime traces.

## Implementation linkage

- Graph model: `crates/bijux-dag-core/src/lib.rs` (`Node` and `PortRef` structures).
- Topology and edge linkage: `crates/bijux-dag-core/src/graph/topology.rs` and `crates/bijux-dag-core/src/graph/edge.rs`.
- Validation guards: `crates/bijux-dag-core/src/pipeline/validate.rs`.

## Identity invariants

- Edges reference nodes through `from.node_id` and `to.node_id`.
- Planner lowering preserves `node.id` as execution-plan node identity.
- Runtime trace and outputs index preserve node identity linkage.

## Relationship to fingerprints

- `node.id` is one of the direct contributors to node fingerprints.
- Node fingerprints are deterministic over canonical node semantics.
- Graph identity is derived from canonical graph bytes and includes node semantics and edges.

## Related contracts

- `docs/spec/GRAPH_IDENTITY_CONTRACT.md`
- `docs/spec/FINGERPRINTS_v0.1.md`
- `docs/spec/PLANNER_CONTRACT.md`
- `docs/spec/RUN_ARTIFACT_SPEC_v0.1.md`
