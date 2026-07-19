---
title: Execution Mode Responsibilities
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Execution Mode Responsibilities

Execution modes in `bijux-dag` must state which runtime path is actually
implemented, which path is modeled, and where deployment or release framing
must stop short of broader claims.

## Current Execution Modes

- local execution is the core implemented runtime path
- container execution is an implemented local engine-mediated lane with mount,
  environment, and output-boundary enforcement
- kubernetes execution is an implemented batch backend for container nodes
  through Kubernetes Jobs
- slurm execution is an implemented shared-filesystem batch backend through
  `sbatch` and `sacct`
- remote-worker and generic hpc surfaces remain modeled or unreleased rather
  than stable operator promises

## Responsibility Split

- runtime contracts under `docs/spec/` define mode semantics and hard
  boundaries
- architecture pages define which subsystem owns the behavior
- operations pages define deployment prerequisites and operator expectations
- release-boundary pages define whether a mode is stable, internal, simulated,
  or unreleased

## Proof Surfaces

- `docs/spec/CONTAINER_EXECUTION_CONTRACT.md`
- `docs/spec/REMOTE_EXECUTION_MODEL.md`
- `docs/spec/BATCH_EXECUTION_MODEL.md`
- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/kubernetes_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/slurm_execution_contracts.rs`

## Detailed Walkthrough

Use [Reference: Execution Mode Responsibilities](execution-mode-responsibilities.md)
for the narrower mode-by-mode proof inventory.
