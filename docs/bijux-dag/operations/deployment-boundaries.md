---
title: Deployment Boundaries
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Deployment Boundaries

Deployment boundaries define what must be consistent across environments to keep
DAG run meaning stable.

## Visual Summary

```mermaid
flowchart LR
    graph_def[graph definition] --> boundary[deployment boundary]
    runtime[runtime config] --> boundary
    storage[artifact backend] --> boundary
    boundary --> reproducibility[replay and diff comparability]
```

## Boundary Layers

- graph/schema/version compatibility
- runtime feature set and execution mode
- artifact storage backend semantics
- environment variables and secrets exposure model

## Enforcement Rules

- deployment metadata must be discoverable from run artifacts
- capability downgrades must be surfaced as explicit fidelity changes
- cross-boundary comparisons must include environment context

## Code Anchors

- `crates/bijux-dag-runtime/src/config.rs`
- `crates/bijux-dag-artifacts/src/storage/`
- `crates/bijux-dag-app/src/routes/replay_routes.rs`

## Next Reads

- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
- [Release and Versioning](release-and-versioning.md)
- [Security and Safety](security-and-safety.md)
