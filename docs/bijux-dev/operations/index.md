---
title: Dev Operations
audience: maintainers
type: section-index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Dev Operations

This section documents day-to-day maintainer operations for running governance
commands, collecting evidence, and coordinating release readiness.

These pages explain the operational side of the maintainer package. For the
root command surface itself, use [makes](../makes/index.md). For hosted
automation entrypoints, use [gh-workflows](../gh-workflows/index.md).

## Visual Summary

```mermaid
flowchart LR
    setup[toolchain setup] --> commands[command surface]
    commands --> gates[repository gates]
    gates --> evidence[evidence collection]
    evidence --> diagnostics[diagnostics and reporting]
    diagnostics --> release[release operations]
```

## Pages In This Section

- [Toolchain Setup](toolchain-setup.md)
- [Command Surface](command-surface.md)
- [Repository Gates](repository-gates.md)
- [Evidence Collection](evidence-collection.md)
- [Diagnostics and Reporting](diagnostics-and-reporting.md)
- [Docs Operations](docs-operations.md)
- [CI and Automation](ci-and-automation.md)
- [Incident Response](incident-response.md)
- [Release Operations](release-operations.md)

## Related Maintainer Sections

- [Dev Governance](../governance/index.md)
- [makes](../makes/index.md)
- [gh-workflows](../gh-workflows/index.md)
