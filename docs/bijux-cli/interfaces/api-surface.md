---
title: API Surface
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# API Surface

The `src/api` module tree is the intentional Rust-facing surface for invoking
runtime behavior without importing deep internal module paths.

## Visual Summary

```mermaid
flowchart LR
    callers["callers and tests"] --> api_mod["api facade modules"]
    api_mod --> runtime["runtime run_app run_cli_from_env"]
    api_mod --> routing["parser and routing facade"]
    api_mod --> diagnostics["diagnostics and install facade"]
    api_mod --> repl["repl facade and contracts"]
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

## Next Reads

- [Public Imports](public-imports.md)
- [Data Contracts](data-contracts.md)
- [Code Navigation](../architecture/code-navigation.md)
