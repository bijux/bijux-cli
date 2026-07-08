---
title: Deployment Boundaries
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Deployment Boundaries

Deployment boundaries clarify what `bijux-cli` can guarantee in different host
and packaging contexts, and where responsibility shifts to adjacent tooling.

## Visual Summary

```mermaid
flowchart TB
    runtime["bijux-cli runtime"] --> host["supported host expectations"]
    runtime --> packaging["binary and package channels"]
    runtime --> adjacencies["known tool delegation boundaries"]
    adjacencies --> external["external runtime ownership"]
```

## Boundary Areas

- host environment assumptions for command and completion behavior
- package/channel alignment for runtime version identity
- delegated known-tool route handoff boundaries
- state directory and plugin path ownership boundaries

## Code Anchors

- `crates/bijux-cli/src/interface/cli/dispatch/delegation.rs`
- `crates/bijux-cli/src/features/install/compatibility.rs`
- `crates/bijux-cli/src/features/install/paths.rs`
- `crates/bijux-cli/src/features/diagnostics/state_paths.rs`

## Boundary Rules

- document host limitations clearly and keep them current
- treat delegated-route failures as observable operational errors
- avoid implicit cross-package assumptions in CLI-only release notes

## Next Reads

- [Repository Fit](../../foundation/repository-fit.md)
- [Integration Seams](../../architecture/integration-seams.md)
- [Risk Register](../../quality/risk-register.md)
