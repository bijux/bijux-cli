---
title: Command Surface
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Command Surface

This page explains the command entrypoints that power repository proof work.

`bijux-dev-cli` carries the general repository workflow. `bijux-dev-dag` carries
the DAG-specific verification and release surfaces that sit beside it.

## Command Map

```mermaid
flowchart LR
    maintainer["repository maintainer"] --> dev_cli["bijux-dev-cli"]
    maintainer --> dev_dag["bijux-dev-dag"]
    dev_cli --> verify["workspace checks and reports"]
    dev_dag --> evidence["DAG evidence and release checks"]
```

## Command Families

- validation commands for repository and contract checks
- report commands for architecture, coverage, and evidence status
- release commands for readiness and compatibility workflows
- documentation and governance commands for handbook integrity

## Command Design Rules

- commands must return actionable diagnostics
- machine-readable output must remain stable for automation
- command semantics must map to explicit ownership in code and docs

## Reading Rule

Use this page when you know the repository needs a maintainer command but have
not yet decided which entrypoint owns the job. Move to Diagnostics, Release
Operations, or Contract Governance once the command family is clear.

## Code Anchors

- `crates/bijux-dev/src/cli.rs`
- `crates/bijux-dev/src/commands/mod.rs`
- `crates/bijux-dev/src/bin/bijux-dev-cli.rs`
- `crates/bijux-dev/src/main.rs`

## Next Reads

- [Diagnostics and Reporting](diagnostics-and-reporting.md)
- [Contract Governance](../governance/contract-governance.md)
- [Core Maintainer Control Plane](../../bijux-core/architecture/maintainer-control-plane.md)
