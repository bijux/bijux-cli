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
- kubernetes execution: modeled contract surface, not implemented
- hpc execution: modeled contract surface, not implemented

## Responsibility split

- specs under `docs/spec/` define contract-level behavior and constraints
- deployment operations docs explain operator-facing environment boundaries
- runtime tests define executable proof for mode classification and handoff
  semantics

## Primary proof

- `crates/bijux-dag-runtime/tests/adapter_runtime_contracts.rs`
- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`
- `docs/spec/CONTAINER_EXECUTION_CONTRACT.md`
- `docs/spec/REMOTE_EXECUTION_MODEL.md`
