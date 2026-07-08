---
title: Execution Mode Responsibilities
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Execution Mode Responsibilities

Execution modes in `bijux-dag` must describe what the runtime actually owns,
what remains modeled, and where policy stops short of stronger isolation
claims.

## Current mode boundaries

- local execution: implemented runtime path
- container execution: implemented local engine-backed path with explicit
  constraints
- kubernetes execution: modeled runtime lane with simulated Job and pod
  semantics, resource and deadline mapping, workspace transfer contracts, and
  pod phase mapping
- slurm execution: implemented shared-filesystem runtime lane with `sbatch`
  submission, `sacct` polling, batch evidence retention, and worker re-entry
  through the same retained run directory
- generic hpc execution: modeled runtime lane beyond the concrete shared-
  filesystem SLURM backend

## Responsibility split

- specs under `docs/spec/` define contract-level behavior and constraints
- deployment operations docs explain operator-facing environment boundaries
- runtime tests define executable proof for mode classification and handoff
  semantics

## Primary proof

- `crates/bijux-dag-runtime/tests/adapter_runtime_contracts.rs`
- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/kubernetes_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/kubernetes_backend_contracts.rs`
- `crates/bijux-dag-runtime/tests/slurm_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/slurm_backend_contracts.rs`
- `docs/spec/CONTAINER_EXECUTION_CONTRACT.md`
- `docs/spec/REMOTE_EXECUTION_MODEL.md`
