---
title: API Surface
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# API Surface

DAG Rust APIs are exposed through crate roots and explicit re-exports, allowing
consumers to use semantic, runtime, and artifact capabilities without importing
internal module paths.

## Visual Summary

```mermaid
sequenceDiagram
    participant Client as Rust caller
    participant API as Crate-root exports
    participant Core as Core/runtime/artifact logic
    participant Contracts as Data contracts

    Client->>API: call public function
    API->>Contracts: validate request shape
    API->>Core: invoke operation
    Core-->>API: result or typed error
    API->>Contracts: format response payload
    API-->>Client: stable response surface
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

## Next Reads

- [Public Imports](public-imports.md)
- [Data Contracts](data-contracts.md)
- [Code Navigation](../architecture/code-navigation.md)
