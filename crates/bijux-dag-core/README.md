# bijux-dag-core

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-core?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-core)
[![Rust docs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/) [![bijux-dag-core docs](https://img.shields.io/badge/docs-core-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-core/)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-core` is the deterministic graph kernel behind `bijux-dag`.
It handles graph truth: parsing, validation, canonicalization, topology,
identity, reference resolution, and planner lowering.

`bijux-dag` v0.4.1 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate provides
the explicit graph-contract half of that product promise.

## Release Status

- public crate on the `v0.4.1` DAG release line
- pure kernel layer for DAG authoring, validation, and identity work

## What It Provides

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
[`evidence/dag/authoring/examples/release-note-bundle.dag.json`](https://github.com/bijux/bijux-core/blob/main/evidence/dag/authoring/examples/release-note-bundle.dag.json).

For the repository-backed authoring example that binds a graph-owned enum input
into a real branch decision surface, use
[`evidence/dag/authoring/examples/audience-branch-bulletin.dag.json`](https://github.com/bijux/bijux-core/blob/main/evidence/dag/authoring/examples/audience-branch-bulletin.dag.json).

For the repository-backed authoring example that binds graph-owned path inputs
into a retryable compliance gate and a repairable publication boundary, use
[`evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json`](https://github.com/bijux/bijux-core/blob/main/evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json).

For the operator-facing explanation of how graph identity relates to plan,
execution, cache, and replay identity after this crate lowers a graph into
deterministic execution surfaces, use
[Reproducibility Model](https://bijux.io/bijux-core/bijux-dag/interfaces/reproducibility-model/).

Repository-owned schedule and backfill authoring examples also live under
`evidence/dag/authoring/examples/`, but they remain internal workflow lanes
rather than part of the default public `v0.4.x` package story.

## Internal Documentation

- [`ARCHITECTURE.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-core/docs/ARCHITECTURE.md): pure-kernel data flow, source
  boundaries, dependency direction, and extension decisions.
- [`CONTRACTS.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-core/docs/CONTRACTS.md): owned graph semantics, purity,
  validation, identity, and stability contracts.
- [`GRAPH_MODEL.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-core/docs/GRAPH_MODEL.md): graph, node, reference, output,
  edge, trigger, composition, and expansion model.
- [`IDENTITY_AND_VALIDATION.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-core/docs/IDENTITY_AND_VALIDATION.md): strict
  parsing, diagnostics, canonicalization, fingerprints, and compatibility.
- [`PLANNING_AND_PUBLIC_API.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-core/docs/PLANNING_AND_PUBLIC_API.md): planner
  lowering, compile helpers, stable exports, and runtime handoff.
- [`SERIALIZATION_AND_EVOLUTION.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-core/docs/SERIALIZATION_AND_EVOLUTION.md):
  strict input shape, version ownership, canonical compatibility, identity,
  and schema-change procedure.

## Related links

- [Crate contracts](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-core/docs/CONTRACTS.md)
- [Crate changelog](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-core/CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-core/)
