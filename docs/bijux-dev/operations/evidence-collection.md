---
title: Evidence Collection
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Evidence Collection

This page explains how `bijux-core` turns verification output into something
worth reviewing later.

The repository does not treat raw command output as enough on its own. Useful
evidence has to stay tied to the exact commit, the exact command, and the exact
decision it supports.

## Evidence Flow

```mermaid
flowchart LR
    tests["tests and suite runs"] --> outputs["command outputs"]
    outputs --> reports["evidence reports"]
    reports --> review["review and release decisions"]
```

## Evidence Sources

- test outputs from workspace and program suites
- report generators in `bijux-dev` command binaries
- contract files and schema checks under `contracts/`
- docs-check outputs for handbook publish integrity

## Collection Rules

- evidence must reference the exact commit under review
- generated reports are auditable artifacts, not policy replacements
- missing evidence is treated as unresolved risk, not assumed success

## Reading Rule

Use this page when the work is already verified once but the proof still needs
to be carried into review, release, or follow-up decisions.

## Code Anchors

- `crates/bijux-dev/src/commands/evidence_registry.rs`
- `crates/bijux-dev/src/commands/evidence_control_plane.rs`
- `crates/bijux-dev/src/report/`

## Next Reads

- [Diagnostics and Reporting](diagnostics-and-reporting.md)
- [Change Control](../governance/change-control.md)
- [Risk and Exceptions](../../bijux-core/governance/risk-and-exceptions.md)
