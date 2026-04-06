---
title: Public Imports
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Public Imports

Prefer crate-root exports for DAG integrations so code remains aligned with
intended ownership boundaries.

## Visual Summary

```mermaid
flowchart TB
    caller["rust caller"] --> core["bijux_dag_core::*"]
    caller --> runtime["bijux_dag_runtime::*"]
    caller --> artifacts["bijux_dag_artifacts::*"]
    caller -.avoid deep internal paths.-> internals["internal module paths"]
```

## Preferred Imports

- `bijux_dag_core::{Graph, GraphError, parse_graph_strict, lower_graph_to_execution_plan}`
- `bijux_dag_runtime::{Runtime, RuntimeConfig, build_plan}`
- `bijux_dag_artifacts::{RunDir, verify_run_dir, write_outputs_index}`
- `bijux_dag_app::{dag_command, dag_run}` for CLI wiring integrations

## Code Anchors

- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-runtime/src/lib.rs`
- `crates/bijux-dag-artifacts/src/lib.rs`
- `crates/bijux-dag-app/src/lib.rs`

## Next Reads

- [API Surface](api-surface.md)
- [Dependency Governance](../quality/dependency-governance.md)
- [Compatibility Commitments](compatibility-commitments.md)
