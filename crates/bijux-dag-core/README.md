# bijux-dag-core

`bijux-dag-core` is the deterministic kernel behind the `bijux-dag` product.
It owns graph truth: parsing, validation, canonicalization, topology,
identity, reference resolution, and planner lowering.

## Release Status

- public crate on the `v0.4.0` DAG release line
- pure kernel layer for DAG authoring, validation, and identity work

## What This Crate Owns

- strict graph parsing and validation with stable diagnostics
- deterministic graph canonicalization and topology ordering
- graph and node fingerprinting primitives
- planner-lowering helpers used by runtime and app layers
- command-template and graph-input resolution rules that let shell and
  container nodes bind stable params without runtime guesswork

Choose this crate when you need a pure Rust dependency for DAG authoring,
inspection, validation, or identity work without pulling in runtime execution
or command-layer concerns.

## What It Does Not Own

- adapter implementations or runtime scheduling
- command parsing, rendering, or CLI routing
- filesystem, process, or wall-clock side effects

## Source Layout

- `src/graph`: graph model, parsing, and semantic validation
- `src/pipeline`: compile-path helpers and validation entrypoints
- `src/analysis`: fingerprints, equivalence inputs, and deterministic analysis
- `src/planner`: planner-lowering primitives
- `src/build` and `src/contracts`: build-facing wrappers and typed contracts

## Reach For Another Crate When

- you need actual run execution or replay behavior:
  `bijux-dag-runtime`
- you need operator-facing command orchestration:
  `bijux-dag-app`
- you need persisted evidence models:
  `bijux-dag-artifacts`

## Representative Example

For the repository-backed authoring example that binds a graph-owned label into
a real container command surface, use
[`evidence/dag/authoring/examples/release-note-bundle.dag.json`](../../evidence/dag/authoring/examples/release-note-bundle.dag.json).

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-core/)
