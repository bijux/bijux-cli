---
title: Execution Model
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Execution Model

DAG execution converts validated graph semantics into node outcomes and artifact
evidence under explicit scheduler and policy controls.

## Visual Summary

```mermaid
flowchart LR
    parse["parse and validate graph"] --> plan["lower to execution plan"]
    plan --> schedule["scheduler selects ready nodes"]
    schedule --> execute["engine invokes adapters"]
    execute --> persist["write run and artifact evidence"]
    persist --> classify["replay and diff classification surfaces"]
```

## Execution Stages

1. parse, validate, canonicalize graph input
2. lower to runtime execution plan
3. scheduler computes dependency-correct frontier
4. engine executes nodes through adapter boundaries
5. runtime writes node traces and run outputs
6. replay/diff services consume persisted evidence

## Code Anchors

- `crates/bijux-dag-core/src/pipeline/parse.rs`
- `crates/bijux-dag-core/src/planner/planner.rs`
- `crates/bijux-dag-runtime/src/runtime_core/execution/scheduler.rs`
- `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs`
- `crates/bijux-dag-artifacts/src/lib.rs`

## Next Reads

- [State and Persistence](state-and-persistence.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Failure Recovery](../operations/failure-recovery.md)
