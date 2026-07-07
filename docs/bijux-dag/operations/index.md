---
title: DAG Operations
audience: operators
type: section-index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# DAG Operations

DAG operations focus on repeatable execution, artifact evidence, and predictable
recovery under change.

## Section Map

```mermaid
flowchart LR
    operations["DAG operations"] --> setup["installation and setup"]
    operations --> run["common run workflows"]
    operations --> observe["observability and diagnostics"]
    operations --> recover["failure recovery"]
    operations --> release["release and versioning"]
```

## Operating Priorities

- prefer deterministic runs over convenience shortcuts
- preserve evidence before remediation actions
- diagnose with command output and artifacts, not assumptions
- keep release policy and runtime behavior synchronized

## Core Runbook Pages

- [Installation and Setup](installation-and-setup.md)
- [Local Development](local-development.md)
- [Common Workflows](common-workflows.md)
- [Observability and Diagnostics](observability-and-diagnostics.md)
- [Failure Recovery](failure-recovery.md)

## Boundary and Governance Pages

- [Deployment Boundaries](deployment-boundaries.md)
- [Branching Bulletin Workflow](guides/branching-bulletin-workflow.md)
- [CI Integration](guides/ci-integration.md)
- [Compliance-Gated Bulletin Workflow](guides/compliance-gated-bulletin-workflow.md)
- [Container Packaging Workflow](guides/container-packaging-workflow.md)
- [Data Pipeline Workflow](guides/data-pipeline-workflow.md)
- [File Processing Workflow](guides/file-processing-workflow.md)
- [First Hour With Bijux Dag](guides/first-hour-with-bijux-dag.md)
- [Historical Catalog Backfill Workflow](guides/historical-catalog-backfill-workflow.md)
- [Scheduled Catalog Refresh Workflow](guides/scheduled-catalog-refresh-workflow.md)
- [Trust Boundaries](reference/trust-boundaries.md)
- [Performance and Scaling](performance-and-scaling.md)
- [Release and Versioning](release-and-versioning.md)
- [Security and Safety](security-and-safety.md)

## Cross References

- [Operator Workflows](../interfaces/operator-workflows.md)
- [Change Validation](../quality/change-validation.md)

## Reading Rule

Use this section when the question is about executing, diagnosing, or recovering
graph runs in practice. Move back to Interfaces or Quality when the next
question is about contracts or proof rather than operations.
