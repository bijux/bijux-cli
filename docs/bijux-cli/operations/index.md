---
title: Operations
audience: mixed
type: index
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# CLI Operations

The operations section describes how to run, validate, diagnose, and release
`bijux-cli` in daily engineering and automation workflows.

## Visual Summary

```mermaid
flowchart LR
    setup["installation and setup"] --> dev["local development"]
    dev --> workflows["common workflows"]
    workflows --> diagnostics["observability and diagnostics"]
    diagnostics --> release["release and recovery practices"]
```

## Operational Scope

- installation and first-run verification
- local build/test and command iteration loops
- day-to-day command workflows for operators
- diagnostics and telemetry collection patterns
- release, security, and deployment boundary practices

## Code Anchors

- `crates/bijux-cli/src/features/install/`
- `crates/bijux-cli/src/interface/cli/handlers/cli.rs`
- `crates/bijux-cli/src/shared/telemetry.rs`
- `crates/bijux-cli/tests/integration/`

## Pages In This Section

- [Installation and Setup](installation-and-setup.md)
- [Local Development](local-development.md)
- [Common Workflows](common-workflows.md)
- [Observability and Diagnostics](observability-and-diagnostics.md)
- [Performance and Scaling](performance-and-scaling.md)
- [Failure Recovery](failure-recovery.md)
- [Release and Versioning](release-and-versioning.md)
- [Security and Safety](security-and-safety.md)
- [Deployment Boundaries](deployment-boundaries.md)
