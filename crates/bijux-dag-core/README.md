# bijux-dag-core

`bijux-dag-core` is the deterministic graph kernel behind `bijux-dag`.
It handles graph truth: parsing, validation, canonicalization, topology,
identity, reference resolution, and planner lowering.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate provides
the explicit graph-contract half of that product promise.

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
- branch contracts, conditional-edge validation, and trigger-rule constraints
  that keep selected and skipped lanes deterministic

Use this crate when you need a pure Rust dependency for DAG authoring,
inspection, validation, or identity work without pulling in runtime execution
or command-layer concerns.

## What It Does Not Own

- adapter implementations or runtime scheduling
- command parsing, rendering, or CLI routing
- filesystem, process, or wall-clock side effects

## Public Rust Surface

- browse docs.rs through `bijux_dag_core::stable` for the long-lived graph
  compatibility lane
- use `bijux_dag_core::prelude` for parse, validate, canonicalize, and plan
  workflows
- use focused crate-root imports only when you already know the exact graph
  item you need
- broad compatibility re-exports remain callable for repository-owned support
  work, but stay hidden from the primary docs.rs lane

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

## Good Fit

- building DAG authoring or validation tooling in Rust
- computing canonical graph identity before any execution side effects
- lowering validated graphs into deterministic planner inputs
- reusing Bijux graph semantics without depending on the CLI or runtime

## Representative Examples

For the repository-backed authoring example that binds a graph-owned label into
a real container command surface, use
[`evidence/dag/authoring/examples/release-note-bundle.dag.json`](../../evidence/dag/authoring/examples/release-note-bundle.dag.json).

For the repository-backed authoring example that binds a graph-owned enum input
into a real branch decision surface, use
[`evidence/dag/authoring/examples/audience-branch-bulletin.dag.json`](../../evidence/dag/authoring/examples/audience-branch-bulletin.dag.json).

For the repository-backed authoring example that binds graph-owned path inputs
into a retryable compliance gate and a repairable publication boundary, use
[`evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json`](../../evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json).

For the repository-backed authoring example that binds a schedule-owned
timestamp into a required integer graph input and a promotable publication
output, use
[`evidence/dag/authoring/examples/scheduled-catalog-refresh.dag.json`](../../evidence/dag/authoring/examples/scheduled-catalog-refresh.dag.json).

For the repository-backed authoring example that binds requested slots plus
backfill-owned window metadata into a real retained publication surface, use
[`evidence/dag/authoring/examples/historical-catalog-backfill.dag.json`](../../evidence/dag/authoring/examples/historical-catalog-backfill.dag.json).

For the operator-facing explanation of how graph identity relates to plan,
execution, cache, and replay identity after this crate lowers a graph into
deterministic execution surfaces, use
[Reproducibility Model](../../docs/bijux-dag/interfaces/reference/reproducibility-model.md).

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-core/)
