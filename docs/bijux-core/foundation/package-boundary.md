---
title: Package Boundary
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-06
---

# Package Boundary

`bijux-core` ships two public release families and several repository-internal
support crates. This page is the canonical answer to one question: which
workspace packages are published, and which ones stay private?

The contract source for this page is
`contracts/foundation/workspace_package_boundary.v1.json`.

## Release Status Table

| Package | Product family | Release status | Purpose |
| --- | --- | --- | --- |
| `bijux-cli` | `bijux-cli` | public | operator-facing command runtime for automation, plugin routing, interactive workflows, and structured output |
| `bijux-cli-python` | `bijux-cli` | private | Python packaging bridge for the `bijux` command runtime |
| `bijux-dag-core` | `bijux-dag` | public | deterministic DAG kernel for graph parsing, validation, canonicalization, planning, and identity |
| `bijux-dag-artifacts` | `bijux-dag` | public | artifact identity, persistence, and integrity primitives for DAG run evidence |
| `bijux-dag-runtime` | `bijux-dag` | public | execution kernel, scheduler policy, replay decisions, and runtime state transitions for DAG runs |
| `bijux-dag-app` | `bijux-dag` | public | DAG command orchestration, inspection, replay, and verification response shaping |
| `bijux-dag-cli` | `bijux-dag` | public | thin `bijux-dag` executable wrapper over the DAG application surface |
| `bijux-dag-testkit` | `bijux-dag` | private | shared deterministic fixtures, fake adapters, and DAG assertions for repository tests |
| `bijux-dev` | `maintainer` | private | maintainer control plane for release governance, repository evidence, and diagnostics |

## crates.io Publication Order

The canonical crates.io publish order is:

1. `bijux-dag-core`
2. `bijux-dag-artifacts`
3. `bijux-dag-runtime`
4. `bijux-dag-app`
5. `bijux-dag-cli`
6. `bijux-cli`

The DAG crate family publishes first so dependency edges are satisfied before
the separate `bijux` runtime crate is released.

## Reading Rules

- treat `public` as the supported crates.io publication boundary
- treat `private` as repository-owned support code that must keep `publish = false`
- do not add runtime or build-time dependencies from public crates onto private crates
- update this page and `contracts/foundation/workspace_package_boundary.v1.json` together when package publication intent changes

## Next Reads

- [Package Map](package-map.md)
- [Repository Packages](../packages/index.md)
- [Release Operations](../../bijux-dev/operations/release-operations.md)
