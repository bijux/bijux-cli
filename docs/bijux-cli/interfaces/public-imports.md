---
title: Public Imports
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Public Imports

This page records preferred import paths for Rust callers that depend on
`bijux-cli` runtime behavior.

## Visual Summary

```mermaid
flowchart TB
    caller["Rust caller"] --> api["use crate::api facade"]
    api --> runtime["runtime interfaces"]
    api --> parser["parser and routing interfaces"]
    api --> output["output interfaces"]
    caller -.avoid direct reliance on.-> internals["deep internal module paths"]
```

## Preferred Imports

- `bijux_cli::api::runtime::*`
- `bijux_cli::api::parser::*`
- `bijux_cli::api::routing::*`
- `bijux_cli::api::output::*`
- `bijux_cli::api::diagnostics::*`
- `bijux_cli::api::repl::*`

## Import Guidance

- import from `api` when building tools or tests against runtime behavior
- avoid importing private module internals that are not part of facade intent
- when new facade exports are added, document them in this page

## Code Anchors

- `crates/bijux-cli/src/api/mod.rs`
- `crates/bijux-cli/src/lib.rs`
- `crates/bijux-cli/Cargo.toml`

## Next Reads

- [API Surface](api-surface.md)
- [Compatibility Commitments](compatibility-commitments.md)
- [Dependency Governance](../quality/dependency-governance.md)
