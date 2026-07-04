---
title: Common Workflows
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Common Workflows

This page captures the DAG workflow path people follow most often.

The sequence matters because DAG work is less about one command and more about
moving safely from validation to evidence-backed decisions.

## Workflow Map

```mermaid
flowchart TD
    define["prepare graph and inputs"] --> validate["validate"]
    validate --> execute["run"]
    execute --> inspect["inspect status and artifacts"]
    inspect --> compare["replay and diff"]
    compare --> decide["promote or investigate"]
```

## Workflow Catalog

- preflight validation for graph and config correctness
- execution workflow for run creation and status tracking
- reproducibility workflow for replay confirmation
- change-attribution workflow for semantic diff explanations

## Canonical Command Path

```bash
bijux-dag validate ./pipelines/main.dag.json
bijux-dag run ./pipelines/main.dag.json --out ./runs/proposed
bijux-dag status ./runs/proposed/latest
bijux-dag replay ./runs/proposed/latest --out ./runs/replay
bijux-dag diff ./runs/reference/latest ./runs/proposed/latest --mode semantic --explain
```

## Code Anchors

- `crates/bijux-dag-app/src/routes/run_routes.rs`
- `crates/bijux-dag-app/src/routes/status_routes.rs`
- `crates/bijux-dag-app/src/routes/inspect_routes.rs`

## Promotion Criteria

- run completed with expected fidelity level
- required artifact evidence present and verifiable
- drift either absent or explicitly approved

## Reading Rule

Use this page when the DAG commands are already familiar but the correct
operator sequence is still unclear.

## Next Reads

- [Failure Recovery](failure-recovery.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Review Checklist](../quality/review-checklist.md)
