# bijux-dag-core

`bijux-dag-core` is the deterministic kernel behind the `bijux-dag` product.
It owns graph truth: parsing, validation, canonicalization, topology,
identity, reference resolution, and planner lowering.

## What this crate provides

- Strict graph parsing and validation with stable diagnostics.
- Deterministic graph canonicalization and topology ordering.
- Graph and node fingerprinting primitives.
- Planner-lowering helpers used by runtime and app layers.

Choose this crate when you need a pure Rust dependency for DAG authoring,
inspection, validation, or identity work without pulling in runtime execution or
command-layer concerns.

## Deliberate boundaries

This crate stays pure and deterministic. It does not own:

- adapter implementations or runtime scheduling,
- command parsing, rendering, or CLI routing,
- filesystem, process, or wall-clock side effects.

## Source layout

- `src/graph`: graph model, parsing, and semantic validation
- `src/pipeline`: compile-path helpers and validation entrypoints
- `src/analysis`: fingerprints, equivalence inputs, and deterministic analysis
- `src/planner`: planner-lowering primitives
- `src/build` and `src/contracts`: build-facing wrappers and typed contracts

## Reach for another crate when

- you need actual run execution or replay behavior:
  `bijux-dag-runtime`
- you need operator-facing command orchestration:
  `bijux-dag-app`
- you need persisted evidence models:
  `bijux-dag-artifacts`

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-core/)
