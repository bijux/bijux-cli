---
title: DAG Operations
audience: operators
type: section-index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# DAG Operations

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles.
The [Replay Contract](../../spec/REPLAY_CONTRACT.md) defines the replay authority.

Use this section when you need to run real DAG workflows, inspect retained
evidence, recover from failures, or understand the operational boundary of the
released local-first product.

DAG operations focus on repeatable execution, retained artifacts, and
predictable recovery under change.

## Start With The Situation You Have

| If you need to... | Open this page |
| --- | --- |
| get from checkout to a real run quickly | [First-Run Tutorial](first-run-tutorial.md) |
| install the tool and verify the environment | [Installation and Setup](installation-and-setup.md) |
| run the normal local workflow loop | [Common Workflows](common-workflows.md) |
| inspect failures, traces, and retained evidence | [Observability and Diagnostics](observability-and-diagnostics.md) |
| recover from runtime or workflow failures | [Failure Recovery](failure-recovery.md) |
| understand release boundaries and what the shipped product claims today | [v0.4.0 Release Notes](v0-4-0-release-notes.md) |
| understand runtime limits before deployment or isolation work | [Deployment Boundaries](deployment-boundaries.md) |

## Operating Priorities

- prefer deterministic runs over convenience shortcuts
- preserve evidence before remediation actions
- diagnose with command output and artifacts, not assumptions
- keep release policy and runtime behavior synchronized

## Core Runbook Pages

- [Installation and Setup](installation-and-setup.md)
- [CI Integration](ci-integration.md)
- [First-Run Tutorial](first-run-tutorial.md)
- [Local Development](local-development.md)
- [Common Workflows](common-workflows.md)
- [Observability and Diagnostics](observability-and-diagnostics.md)
- [Failure Recovery](failure-recovery.md)

## Boundary and Governance Pages

- [Deployment Boundaries](deployment-boundaries.md)
- [v0.4.0 Release Notes](v0-4-0-release-notes.md)
- [Branching Bulletin Workflow](branching-bulletin-workflow.md)
- [CI Integration Guide](ci-integration.md)
- [Cache Behavior Workflow](cache-behavior-workflow.md)
- [Compliance-Gated Bulletin Workflow](compliance-gated-bulletin-workflow.md)
- [Container Packaging Workflow](container-packaging-workflow.md)
- [Data Pipeline Workflow](data-pipeline-workflow.md)
- [Evidence-Backed Bulletin Workflow](evidence-backed-bulletin-workflow.md)
- [File Processing Workflow](file-processing-workflow.md)
- [Historical Catalog Backfill Workflow](historical-catalog-backfill-workflow.md)
- [Scheduled Catalog Refresh Workflow](scheduled-catalog-refresh-workflow.md)
- [Execution Security And Isolation](security-isolation-truth.md)
- [Performance and Scaling](performance-and-scaling.md)
- [Release and Versioning](release-and-versioning.md)

## Cross References

- [Executable Examples](../interfaces/runnable-examples.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Change Validation](../quality/change-validation.md)

## Before You Move Deeper

- Stay in this section when the question is how to execute, diagnose, or
  recover real graph runs.
- Move to Interfaces when the next question is what operators or tooling can
  depend on.
- Move to package pages when you already know the issue belongs to one crate
  such as graph truth, runtime policy, or response shaping.
