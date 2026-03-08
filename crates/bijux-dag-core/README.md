# bijux-dag-core

`bijux-dag-core` owns the deterministic DAG kernel: graph types, parsing, validation, canonicalization, topology, identity, resolution, and planner lowering primitives.

## Design intent
The crate is organized around a thin root module and focused service modules:

- `graph/model.rs`: core graph data types
- `graph/canonical.rs`: canonicalization and normalization
- `graph/topology.rs`: deterministic topological ordering
- `pipeline/parse.rs`: strict parse entrypoints
- `pipeline/validate.rs`: validation rules and diagnostics
- `pipeline/resolve.rs`: parameter reference resolution
- `analysis/fingerprint.rs`: graph and node identity
- `planner/planner.rs`: execution-plan lowering

## Boundaries
This crate should stay pure and deterministic. It must not absorb:

- backend adapter implementations
- runtime scheduler or executor orchestration
- CLI routing or formatting concerns
- filesystem or process side effects

See [CONTRACT.md](./CONTRACT.md) for the crate boundary and ownership rules.
