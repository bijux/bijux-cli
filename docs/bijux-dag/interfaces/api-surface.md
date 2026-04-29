---
title: API Surface
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# API Surface

This page explains the Rust-facing DAG surfaces that matter when another crate
needs graph, runtime, or artifact behavior.

The boundary is straightforward: depend on crate-root exports and documented
types, not on internal module paths.

## API Map

```mermaid
flowchart LR
    caller["rust caller"] --> roots["crate-root exports"]
    roots --> core["graph semantics"]
    roots --> runtime["runtime and replay"]
    roots --> artifacts["artifact persistence"]
    roots --> app["command orchestration"]
```

## API Surfaces by Crate

- `bijux-dag-core`: graph model, validation, canonicalization, planner lowering
- `bijux-dag-runtime`: execution engine, scheduling, replay/diff helpers, policy
- `bijux-dag-artifacts`: run-dir lifecycle, artifact integrity, persistence helpers
- `bijux-dag-app`: command orchestration entrypoints (`dag_command`, `dag_run`)

## Code Anchors

- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-runtime/src/lib.rs`
- `crates/bijux-dag-artifacts/src/lib.rs`
- `crates/bijux-dag-app/src/lib.rs`

## API Surface Rules

- favor crate-root exports for external integration code
- avoid coupling to internal modules outside documented contracts
- update interface docs when root export behavior changes

## Reading Rule

Use this page when Rust integration code needs DAG behavior and the main
question is which crate root owns the public call.

## Next Reads

- [Public Imports](public-imports.md)
- [Data Contracts](data-contracts.md)
- [Code Navigation](../architecture/code-navigation.md)
