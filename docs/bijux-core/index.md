---
title: Repository Handbook
audience: mixed
type: index
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Repository Handbook

`bijux-core` repository guidance is split into two stable sections:
architecture and governance.

## Visual Summary

```mermaid
flowchart LR
    core[repository handbook] --> architecture[core architecture section]
    core --> governance[core governance section]
    architecture --> programs[cli dag python dev integration boundaries]
    governance --> policies[release compatibility quality policy]
```

## Section Directory

- [Core Architecture](architecture/index.md)
- [Core Governance](governance/index.md)

## When To Use This Handbook

- when a policy affects more than one program handbook
- when ownership boundaries across crates must be clarified
- when release, compatibility, or validation policy is repository-wide

## Program Handbooks

- [CLI Handbook](../bijux-cli/index.md)
- [DAG Handbook](../bijux-dag/index.md)
- [Maintainer Handbook](../bijux-dev/index.md)
