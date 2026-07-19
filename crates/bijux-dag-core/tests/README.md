# Graph Kernel Contract Tests

These tests protect the pure graph boundary: parsing, validation,
canonicalization, identity, topology, subgraph expansion, and deterministic
planning. They must not require process execution, network access, runtime
state, or artifact storage.

## Coverage

- schema and serde round trips
- validation diagnostics for valid, malformed, and adversarial graphs
- canonical graph identity and mutation sensitivity
- deterministic topology, expansion, map/reduce semantics, and planning
- public Rust imports and prelude stability
- compatibility fixtures under `tests/compat/`
- property and fuzz-derived regression cases for graph shape

Compatibility fixtures are retained only when their serialized bytes or
historical interpretation matter. New semantic cases should prefer small
builders in the test so the relevant invariant is visible.

## Focused Runs

```bash
cargo nextest run -p bijux-dag-core --test canonical_contract
cargo nextest run -p bijux-dag-core --test graph_identity_property_contracts
cargo nextest run -p bijux-dag-core --test planner_contract
```

If a deterministic assertion fails, first inspect ordering, canonical
serialization, and accidental environment input. Do not normalize unstable
output after the fact. I/O in a core test is acceptable only for reading a
governed compatibility fixture.
