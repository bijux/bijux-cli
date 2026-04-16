---
title: Configuration Surface
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Configuration Surface

DAG configuration controls runtime behavior, policy interpretation, cache mode,
and execution materialization choices.

## Visual Summary

```mermaid
flowchart TD
    start[Start configuration load] --> sources[Read sources]
    sources --> defaults[Defaults]
    sources --> file[Config file]
    sources --> env[Environment]
    sources --> flags[CLI flags]
    defaults --> merge[Merge precedence]
    file --> merge
    env --> merge
    flags --> merge
    merge --> validate[Validate]
    validate --> effective[Effective config]
```

## Configuration Inputs

- command flags (`jobs`, `cache`, `cache-dir`, `materialize-inputs`, policy toggles)
- config command surfaces (`config ...`, `policy ...`)
- environment and path resolution inputs where applicable

## Policy-Relevant Controls

- network/env/clock denial flags
- hermetic and clean-env toggles
- select/exclude node targeting
- capability and backend selection where supported

## Code Anchors

- `crates/bijux-dag-app/src/commands/config_surface.rs`
- `crates/bijux-dag-app/src/commands/config_resolution.rs`
- `crates/bijux-dag-app/src/commands/mod.rs`
- `crates/bijux-dag-runtime/src/policy/`

## Configuration Rules

- effective config must be inspectable and explainable
- policy effects must be visible in replay/diff context
- defaults must not silently weaken safety or determinism expectations

## Next Reads

- [State and Persistence](../architecture/state-and-persistence.md)
- [Common Workflows](../operations/common-workflows.md)
- [Change Validation](../quality/change-validation.md)
