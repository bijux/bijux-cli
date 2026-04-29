---
title: Platform Overview
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Platform Overview

`bijux-core` is one workspace with more than one public surface. It keeps CLI
runtime behavior, DAG execution behavior, and repository-health automation in
one governed repository without pretending they are one package.

```mermaid
flowchart LR
    root["bijux-core repository root"] --> cli["CLI runtime surface"]
    root --> dag["DAG execution surface"]
    root --> dev["Maintainer surface"]
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

## Reading Rule

Use this page to understand the top-level split. Move to Package Map when you
need the owning package family, and move to Architecture when the question is
about crate boundaries or dependency direction.

## Next Reads

- [Repository Scope](repository-scope.md)
- [Package Map](package-map.md)
- [Core Architecture](../architecture/index.md)
