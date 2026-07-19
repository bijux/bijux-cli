---
title: Dev Operations
audience: maintainers
type: section-index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Dev Operations

This section documents day-to-day maintainer operations for running governance
commands, collecting evidence, and coordinating release readiness.

These pages explain the operational side of the maintainer package. For the
root command surface itself, use [makes](../makes/index.md). For hosted
automation entrypoints, use [gh-workflows](../gh-workflows/index.md).

## Choose The Operation

| Situation | Start here | Result |
| --- | --- | --- |
| the workstation or toolchain is not trusted yet | [Toolchain Setup](toolchain-setup.md) | verified tools and repository prerequisites |
| the owning command family is unclear | [Command Surface](command-surface.md) | correct maintainer binary and command boundary |
| a change needs local or frozen validation | [Repository Gates](repository-gates.md) | exact gate, commit, result, and artifact location |
| a check passed but reviewable proof is missing | [Evidence Collection](evidence-collection.md) | governed evidence with producer and consumer |
| a gate failed and the failure domain is unclear | [Diagnostics And Reporting](diagnostics-and-reporting.md) | classified defect and owning surface |
| documentation changed | [Documentation Operations](docs-operations.md) | synchronized sources and strict site proof |
| local and hosted results disagree | [CI And Automation](ci-and-automation.md) | environment or workflow attribution |
| release validation failed | [Release Validation Suite](release-validation-suite.md) | failed release claim and evidence owner |
| release artifacts are ready to publish | [Release Operations](release-operations.md) | verified publication sequence and retained proof |
| automation or release state is degraded | [Incident Response](incident-response.md) | contained impact, evidence, and recovery record |

Policy decisions belong in [Dev Governance](../governance/index.md). Local make
entrypoints belong in [makes](../makes/index.md), and hosted triggers belong in
[GitHub Workflows](../gh-workflows/index.md).
