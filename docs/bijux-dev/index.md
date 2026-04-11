---
title: Maintainer Handbook
audience: maintainers
type: index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Maintainer Handbook

`bijux-dev` documents maintainer-only operations and governance for quality
gates, evidence workflows, and release reliability.

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="packages/bijux-dev/">Open the bijux-dev package</a>
<a class="md-button" href="operations/">Open operations</a>
<a class="md-button" href="governance/">Open governance</a>
</div>

## Visual Summary

```mermaid
flowchart TB
    handbook["Maintainer handbook"]
    handbook --> package["bijux-dev package"]
    handbook --> operations["operations section"]
    handbook --> governance["governance section"]
    operations --> evidence["evidence and diagnostics workflows"]
    governance --> policy["quality and contract policy"]
```

## Package Destination

- [`bijux-dev`](packages/bijux-dev/index.md) owns the repository control plane,
  maintainer automation, diagnostics, and release verification flows

## Sections In This Handbook

- [Dev Operations](operations/index.md)
- [Dev Governance](governance/index.md)

## Maintainer Workflow Map

| If you need to... | Start page |
|---|---|
| set up or validate local maintainer tooling | [Toolchain Setup](operations/toolchain-setup.md) |
| run repository gates before merge | [Repository Gates](operations/repository-gates.md) |
| investigate failing verification outputs | [Diagnostics and Reporting](operations/diagnostics-and-reporting.md) |
| handle release or pipeline incidents | [Incident Response](operations/incident-response.md) |
| adjust policy for tests, contracts, or dependencies | [Dev Governance](governance/index.md) |

## Use This Handbook For

- maintainer command workflows and repository gates
- evidence collection and reporting operations
- policy decisions around contracts, dependencies, and security

## Program Handbooks

- [Repository Handbook](../bijux-core/index.md)
- [CLI Handbook](../bijux-cli/index.md)
- [DAG Handbook](../bijux-dag/index.md)

## Decision Boundary

When a question affects runtime behavior seen by end users, switch to the
program handbook (`bijux-cli` or `bijux-dag`) and return here only for
maintainer-specific verification and release workflows.
