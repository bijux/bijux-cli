---
title: Runtime Execution Flow
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Runtime Execution Flow

This page explains how the runtime execution engine stays centralized without
letting cache, trace, and dependency behavior scatter across the codebase.

## Flow Outline

1. materialize declared inputs
2. resolve dependency readiness
3. consult cache proofs
4. execute with retry policy
5. write trace evidence
6. write cache outputs

## Sacred Boundary

The engine in `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs`
must call the sacred hook layer in
`crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs`
rather than directly invoking lower-level helpers.

## Code Anchors

- `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs`
- `crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs`
- `crates/bijux-dag-runtime/tests/sacred_execution_flow_contracts.rs`

## Next Reads

- [Ownership Boundary](../foundation/ownership-boundary.md)
- [Integration Seams](integration-seams.md)
