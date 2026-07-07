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

Use this page when the question is about graph truth before runtime side
effects begin.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| graph model | nodes, edges, resources, metadata, and canonical graph state |
| compile path | parse, validate, resolve, build-contract wrappers, and planner inputs |
| deterministic analysis | fingerprints, semantics, topology, and graph equivalence inputs |
| command templates | graph-input and output-reference rules for shell and container command surfaces |
| branch contracts | semantic branch nodes, conditional-edge validation, and trigger-rule compatibility rules |
| boundary | no scheduler orchestration, CLI routing, or persistence side effects |

## Source Layout

- `crates/bijux-dag-core/src/graph`
- `crates/bijux-dag-core/src/pipeline`
- `crates/bijux-dag-core/src/analysis`
- `crates/bijux-dag-core/src/build`
- `crates/bijux-dag-core/src/planner`
- `crates/bijux-dag-core/src/contracts`

## Open Next

- open the [DAG Handbook](../index.md) for cross-package architecture and operator-facing context
- open [`bijux-dag-runtime`](./bijux-dag-runtime.md) when the question moves from graph truth to execution policy
- open the [Repository Handbook](../../bijux-core/index.md) when the concern crosses into CLI or maintainer policy
- open [Container Packaging Workflow](../operations/guides/container-packaging-workflow.md) when you want the repository example that binds a graph-owned label into a real container command surface
- open [Branching Bulletin Workflow](../operations/guides/branching-bulletin-workflow.md) when you want the repository example that binds a graph-owned enum input into a real branch decision surface

## Code Anchors

- `crates/bijux-dag-core/README.md`
- `crates/bijux-dag-core/CONTRACT.md`
- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-core/src/pipeline/validate.rs`
- `crates/bijux-dag-core/src/planner/planner.rs`

## Review Lens

- graph compilation should remain deterministic and side-effect free
- runtime or CLI concerns should not leak into the kernel layer
- package-local claims should map back to the DAG handbook when they affect the wider stack
