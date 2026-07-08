---
title: Runtime Execution Flow
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Runtime Execution Flow

The runtime keeps scheduling, cache lookup, retry execution, and trace writing
centralized in the engine while routing lower-level work through a guarded hook
layer.

## Flow Outline

1. resolve dependency readiness and trigger rules
2. materialize declared inputs
3. consult cache proofs
4. dispatch ready nodes into the local worker pool
5. execute node attempts with retry policy
6. record trace evidence and terminal status
7. persist cache writes and run summaries

## Sacred Boundary

The engine must use
`crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs`
for shared execution hooks instead of directly bypassing into lower-level
helpers.

## Detailed Walkthrough

Use [Reference: Runtime Execution Flow](reference/runtime-execution-flow.md)
for the full code-anchor map and lifecycle details.
