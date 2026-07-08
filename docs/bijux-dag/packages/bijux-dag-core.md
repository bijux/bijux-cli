---
title: bijux-dag-core Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-07
---

# bijux-dag-core

`bijux-dag-core` owns the deterministic DAG kernel: graph types, parsing,
validation, canonicalization, topology, identity, parameter resolution, and
planner lowering primitives.

At the product level, `bijux-dag` v0.4.0 is a local-first DAG runtime for
reproducible workflows with explicit graph contracts, deterministic execution
records, verified artifacts, cache explanation, and replayable run bundles.
This package owns the explicit graph-contract boundary inside that promise.

Use this page when the question is about graph truth before runtime side
effects begin.

## Reach For This Crate When

- a graph should have validated differently
- node identity, fingerprints, topology, or equivalence look wrong
- you need a pure Rust dependency for authoring, inspection, or validation
  without pulling in runtime execution
- you are deciding whether a bug belongs to graph compilation or to later
  execution behavior

## What It Owns

| Surface | Ownership |
| --- | --- |
| graph model | nodes, edges, resources, metadata, and canonical graph state |
| compile path | parse, validate, resolve, build-contract wrappers, and planner inputs |
| deterministic analysis | fingerprints, semantics, topology, and graph equivalence inputs |
| command templates | graph-input and output-reference rules for shell and container command surfaces |
| branch contracts | semantic branch nodes, conditional-edge validation, and trigger-rule compatibility rules |
| boundary | no scheduler orchestration, CLI routing, or persistence side effects |

## What It Does Not Own

- scheduler decisions, retry behavior, replay policy, and cache reuse belong to
  [`bijux-dag-runtime`](bijux-dag-runtime.md)
- command routing and operator-facing output shaping belong to
  [`bijux-dag-app`](bijux-dag-app.md)
- retained artifact storage authority belongs to `bijux-dag-artifacts`

## Source Layout

- `crates/bijux-dag-core/src/graph`
- `crates/bijux-dag-core/src/pipeline`
- `crates/bijux-dag-core/src/analysis`
- `crates/bijux-dag-core/src/build`
- `crates/bijux-dag-core/src/planner`
- `crates/bijux-dag-core/src/contracts`

## Practical Starting Points

- open the [DAG Handbook](../index.md) for the product story before crate
  boundaries
- open [`bijux-dag-runtime`](bijux-dag-runtime.md) when the question moves
  from graph truth to execution policy
- open [Reproducibility Model](../interfaces/reference/reproducibility-model.md)
  when the question is how graph identity flows into plan, execution, cache,
  and replay identity downstream
- open [Branching Bulletin Workflow](../operations/guides/branching-bulletin-workflow.md)
  when you want a real example of graph-owned branch inputs becoming a retained
  execution decision
- open [Container Packaging Workflow](../operations/guides/container-packaging-workflow.md)
  when you want a graph-owned label and command contract carried into real
  container execution

## Code Anchors

- `crates/bijux-dag-core/README.md`
- `crates/bijux-dag-core/CONTRACT.md`
- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-core/src/pipeline/validate.rs`
- `crates/bijux-dag-core/src/planner/planner.rs`

## Review Focus

- graph compilation should remain deterministic and side-effect free
- runtime or CLI concerns should not leak into the kernel layer
- package-local claims should map back to the DAG handbook when they affect the wider stack
