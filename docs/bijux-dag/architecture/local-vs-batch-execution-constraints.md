---
title: Local Vs Batch Execution Constraints
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Local Vs Batch Execution Constraints

Local execution and batch-oriented execution in `bijux-dag` do not carry the
same ownership model, recovery guarantees, or release posture.

## Boundary Split

- local execution owns the authoritative controller loop, node dispatch, and
  retained artifact production
- kubernetes and slurm backends reuse the same retained evidence model, but
  they add scheduler-specific submission and lifecycle boundaries
- fake batch execution remains a simulation lane for metadata and lifecycle
  reasoning, not a shipped scheduler service

## Constraint Summary

- batch execution must preserve the same run-evidence and node-result contracts
  as local execution
- scheduler identity, retry metadata, and terminal batch evidence are backend
  responsibilities, not planner responsibilities
- restart recovery is not currently claimed as a stable batch guarantee
- generic hpc or public scheduler-service claims remain outside the stable
  operator surface

## Proof Surfaces

- `docs/spec/BATCH_EXECUTION_MODEL.md`
- `crates/bijux-dag-runtime/tests/batch_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs`
- `crates/bijux-dag-runtime/tests/kubernetes_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/slurm_execution_contracts.rs`

## Detailed Walkthrough

Use [Reference: Local Vs Batch Execution Constraints](reference/local-vs-batch-execution-constraints.md)
for the lower-level constraint inventory.
