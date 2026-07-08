---
title: Foundation
audience: mixed
type: index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# DAG Foundation

The foundation section defines what `bijux-dag` is for, what it actually ships
today, which limits still apply, and how the DAG crate family divides
responsibility.

## What This Section Covers

- what `bijux-dag` is built to solve
- what it intentionally does not claim
- responsibility boundaries across DAG crates
- how DAG crates fit inside the workspace publication boundary
- terms used for identity, replay, and diff classification
- principles for safe long-term DAG evolution

## Code Anchors

- `crates/bijux-dag-app/CONTRACT.md`
- `crates/bijux-dag-core/CONTRACT.md`
- `crates/bijux-dag-runtime/CONTRACT.md`
- `crates/bijux-dag-artifacts/CONTRACT.md`

## Pages In This Section

- [Release Boundary](release-boundary.md)
- [Package Overview](package-overview.md)
- [Scope and Non-Goals](scope-and-non-goals.md)
- [Ownership Boundary](ownership-boundary.md)
- [Capability Map](capability-map.md)
- [Domain Language](domain-language.md)
- [Lifecycle Overview](lifecycle-overview.md)
- [Dependencies and Adjacencies](dependencies-and-adjacencies.md)
- [Change Principles](change-principles.md)

## Reading Rule

Start here when the graph system itself is still unclear. Move to Architecture
or Interfaces once the package purpose and boundaries already make sense. Use
[Package Boundary](../../bijux-core/foundation/package-boundary.md) when the
question is whether a crate is public or repository-internal.
