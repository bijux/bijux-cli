---
title: API Surface
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# API Surface

This page explains the Rust-facing entrypoints that matter when code wants CLI
behavior without depending on internal modules.

The public surface is small on purpose: call through the facade, not through
deep implementation paths.

## API Map

```mermaid
flowchart LR
    callers["rust callers"] --> api_mod["api facade"]
    api_mod --> runtime["runtime entrypoints"]
    api_mod --> routing["routing helpers"]
    api_mod --> diagnostics["diagnostics and install helpers"]
    api_mod --> repl["repl contracts"]
```

## Public API Modules

- runtime: process execution entrypoints
- parser/routing: clap intent parsing and route helpers
- output: renderer and emission helpers
- diagnostics: state and route inventory queries
- install/plugins/version/telemetry: runtime integration utilities
- repl: interactive execution contracts and utilities

## Code Anchors

- `crates/bijux-cli/src/api/mod.rs`
- `crates/bijux-cli/src/api/runtime.rs`
- `crates/bijux-cli/src/api/routing.rs`
- `crates/bijux-cli/src/api/repl.rs`
- `crates/bijux-cli/src/api/diagnostics.rs`

## API Surface Rules

- prefer importing from `api` modules over internal implementation paths
- API facades should stay thin and explicit about ownership boundaries
- when facade exports change, update this page and public import guidance

## Reading Rule

Use this page when Rust code needs CLI behavior and the real question is which
facade module owns the call.

## Next Reads

- [Public Imports](public-imports.md)
- [Data Contracts](data-contracts.md)
- [Code Navigation](../architecture/code-navigation.md)
