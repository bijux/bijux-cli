---
title: Maintainer Handbook
audience: maintainers
type: index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Maintainer Handbook

`bijux-dev` documents maintainer-only operations and governance for quality
gates, evidence workflows, and release reliability.

## Visual Summary

```mermaid
flowchart LR
    handbook[maintainer handbook] --> operations[operations section]
    handbook --> governance[governance section]
    operations --> evidence[evidence and diagnostics workflows]
    governance --> policy[quality and contract policy]
```

## Sections In This Handbook

- [Dev Operations](operations/index.md)
- [Dev Governance](governance/index.md)

## Use This Handbook For

- maintainer command workflows and repository gates
- evidence collection and reporting operations
- policy decisions around contracts, dependencies, and security

## Program Handbooks

- [Repository Handbook](../bijux-core/index.md)
- [CLI Handbook](../bijux-cli/index.md)
- [DAG Handbook](../bijux-dag/index.md)
