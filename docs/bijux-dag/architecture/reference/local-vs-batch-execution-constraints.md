---
title: Local Vs Batch Execution Constraints
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Local Vs Batch Execution Constraints

Local execution and modeled batch execution in `bijux-dag` do not carry the
same operational guarantees.

## Current split

- local execution is the implemented runtime path
- fake batch execution is a simulation surface for metadata and lifecycle
  reasoning
- scheduler-specific backends such as Slurm remain outside the implemented
  release boundary

## Constraint summary

- local execution owns real node dispatch and artifact production
- batch simulation owns typed scheduler metadata and lifecycle reasoning only
- restart recovery is intentionally not claimed for the batch surface

## Primary proof

- `crates/bijux-dag-runtime/tests/batch_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs`
- `docs/spec/BATCH_EXECUTION_MODEL.md`

## Next Reads

- [Release Boundary](../../foundation/release-boundary.md)
- [Known Limitations](../../quality/known-limitations.md)
