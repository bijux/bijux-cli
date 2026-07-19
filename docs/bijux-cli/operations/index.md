---
title: Operations
audience: mixed
type: index
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# CLI Operations

Use this section when you are running, supporting, diagnosing, or releasing
`bijux`. It is the operator and maintainer runbook for the command runtime
rather than a description of the public API.

Start here when the question is practical: install it, validate it, recover
from a failure, understand diagnostics, or prepare a release.

## Start With The Situation You Have

| If you need to... | Open this page |
| --- | --- |
| install the CLI and prove the runtime is healthy | [Installation and Setup](installation-and-setup.md) |
| work on the codebase locally | [Local Development](local-development.md) |
| run normal operator flows | [Common Workflows](common-workflows.md) |
| diagnose failures or odd runtime output | [Diagnostics Guide](diagnostics-guide.md) |
| understand observability and telemetry surfaces | [Observability and Diagnostics](observability-and-diagnostics.md) |
| release the CLI or review versioning rules | [Release and Versioning](release-and-versioning.md) |
| understand safety boundaries before deployment | [Security and Safety](security-and-safety.md) |

## What This Section Covers

- installation and first-run verification
- local build/test and command iteration loops
- day-to-day command workflows for operators
- diagnostics and telemetry collection patterns
- release, security, and deployment boundary practices

## Operating Priorities

- prove runtime health with commands and evidence, not assumption
- keep native and bridged entrypoints aligned when diagnosing behavior
- preserve deterministic output and reproducible diagnostics in automation
- treat release and safety documentation as part of the shipped runtime surface

## Code Anchors

- `crates/bijux-cli/src/features/install/`
- `crates/bijux-cli/src/interface/cli/handlers/cli.rs`
- `crates/bijux-cli/src/shared/telemetry.rs`
- `crates/bijux-cli/tests/integration/`

## Pages In This Section

- [Installation and Setup](installation-and-setup.md)
- [Local Development](local-development.md)
- [Common Workflows](common-workflows.md)
- [Diagnostics Guide](diagnostics-guide.md)
- [Observability and Diagnostics](observability-and-diagnostics.md)
- [Performance and Scaling](performance-and-scaling.md)
- [Failure Recovery](failure-recovery.md)
- [Migration Guide](migration-guide.md)
- [Release and Versioning](release-and-versioning.md)
- [Security and Safety](security-and-safety.md)
- [Deployment Boundaries](deployment-boundaries.md)

## Before You Move Deeper

- Stay in this section when the question is how to run or support the CLI in
  practice.
- Move to Interfaces when the next question is about stable caller contracts
  instead of operating behavior.
- Move to package pages when you already know the exact owning crate and need a
  crate-local boundary.
