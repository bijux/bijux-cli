---
title: Foundation
audience: mixed
type: index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# DAG Foundation

The foundation section defines DAG intent and limits before architecture or
command details. Start here when you need the mission, boundaries, vocabulary,
and lifecycle model to make sense before reading execution or interface pages.

## Section Map

```mermaid
flowchart LR
    foundation["DAG foundation"] --> identity["package identity"]
    foundation --> boundaries["ownership boundaries"]
    foundation --> language["domain language"]
    foundation --> lifecycle["graph to replay lifecycle"]
```

## What This Section Covers

- what `bijux-dag` is built to solve
- what it intentionally does not claim
- responsibility boundaries across DAG crates
- terms used for identity, replay, and diff classification
- principles for safe long-term DAG evolution

## Code Anchors

- `crates/bijux-dag-app/CONTRACT.md`
- `crates/bijux-dag-core/CONTRACT.md`
- `crates/bijux-dag-runtime/CONTRACT.md`
- `crates/bijux-dag-artifacts/CONTRACT.md`

## Pages In This Section

- [Package Overview](package-overview.md)
- [Scope and Non-Goals](scope-and-non-goals.md)
- [Ownership Boundary](ownership-boundary.md)
- [Repository Fit](repository-fit.md)
- [Capability Map](capability-map.md)
- [Domain Language](domain-language.md)
- [Lifecycle Overview](lifecycle-overview.md)
- [Dependencies and Adjacencies](dependencies-and-adjacencies.md)
- [Change Principles](change-principles.md)

## Reading Rule

Start here when the graph system itself is still unclear. Move to Architecture
or Interfaces once the package purpose and boundaries already make sense.
