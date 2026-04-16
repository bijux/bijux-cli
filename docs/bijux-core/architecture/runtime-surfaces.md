---
title: Runtime Surfaces
audience: mixed
type: architecture
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Runtime Surfaces

Runtime surfaces define how users and automation interact with CLI and DAG
behavior while preserving shared contracts.

## Visual Summary

```mermaid
flowchart LR
    user[User and automation] --> cli[bijux CLI surface]
    user --> dag[bijux DAG surface]
    cli --> handlers[CLI handlers and plugins]
    dag --> routes[DAG routes]
    routes --> engine[DAG runtime engine]
    handlers --> envelopes[structured output envelopes]
    engine --> envelopes
    envelopes --> contracts[shared output contracts]
```

## Surface Contract

- CLI surfaces provide command routing, plugin lifecycle, and config behavior
- DAG surfaces provide validate, run, replay, diff, status, and inspect flows
- output envelopes must keep machine and human formats semantically aligned
- command behavior changes require corresponding docs and compatibility evidence

## Surface Non-Goals

- no silent alias behavior that bypasses canonical route handling
- no runtime-only shortcuts that produce undocumented output schemas
- no cross-surface drift between machine-readable and human-readable meaning

## Review Questions

1. does this change alter public command meaning?
2. does it change output schema or reason-code vocabulary?
3. are docs and contract tests updated in the same change set?

## Code Anchors

- `crates/bijux-cli/src/main.rs`
- `crates/bijux-dag-cli/src/main.rs`
- `crates/bijux-dag-app/src/routes/mod.rs`
- `crates/bijux-dag-runtime/src/lib.rs`

## Next Reads

- [State and Configuration](state-and-configuration.md)
- [Compatibility and Schema](../governance/compatibility-and-schema.md)
- [Release and Versioning](../governance/release-and-versioning.md)
