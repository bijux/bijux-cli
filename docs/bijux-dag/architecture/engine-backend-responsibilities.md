---
title: Engine Backend Responsibilities
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Engine Backend Responsibilities

The execution engine owns orchestration decisions. Backends own the lifecycle
work required to carry out one node attempt on a specific execution substrate.

## Responsibility Split

- The engine binds a node to a compatible backend, orders lifecycle stages, and
  decides whether the resulting evidence is acceptable for durable run state.
- A backend implements prepare, launch, observe, finalize, and cleanup behavior
  together with backend-specific status, exit-code, and stream reporting.

## Why This Boundary Matters

- Capability mismatches must fail before attempt work starts.
- Undeclared outputs must be rejected by engine-owned evidence checks.
- Backend implementations can vary without changing the run-state contract.

## Detailed Walkthrough

Use [Reference: Engine Backend Responsibilities](reference/engine-backend-responsibilities.md)
for the fuller lifecycle map and code-level anchors.
