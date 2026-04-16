---
title: Platform Overview
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Platform Overview

`bijux-core` is one workspace that publishes and governs more than one
behavioral surface. The repository exists so CLI runtime behavior, DAG
execution behavior, and repository-health automation can evolve together
without pretending they are one package.

```mermaid
flowchart LR
    cli[bijux-cli and bijux-cli-python\noperator runtime surface]
    dag[bijux-dag package family\ngraph execution surface]
    dev[bijux-dev\nrepository-health automation]
    root[Repository root\nshared contracts and rules]

    cli --> root
    dag --> root
    dev --> root
    root -. coordinates shared policy for .-> cli
    root -. coordinates shared policy for .-> dag
```

## What The Repository Is Organizing

- `bijux-cli` owns the operator-facing command runtime and the Python bridge
- `bijux-dag-*` owns graph truth, execution, replay, and artifact semantics
- `bijux-dev` owns repository-health automation, evidence, and release control
- the repository root owns cross-program rules, shared docs, contracts, and
  automation entrypoints

## Why The Split Exists

- command runtime behavior and DAG behavior have different public contracts
- release and documentation evidence must stay reviewable above both products
- maintainer automation should stay explicit instead of leaking into product
  packages

## Next Reads

- [Repository Scope](repository-scope.md)
- [Package Map](package-map.md)
- [Core Architecture](../architecture/index.md)
