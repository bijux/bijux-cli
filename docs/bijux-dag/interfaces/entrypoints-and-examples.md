---
title: Entrypoints and Examples
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-05
---

# Entrypoints and Examples

This page records practical DAG entrypoints for CLI users and Rust integrators.

The CLI examples on this page stay on the stable `v0.4.0` operator surface
from the [Release Boundary](../foundation/release-boundary.md).

## Visual Summary

```mermaid
flowchart LR
    examples[Examples] --> cli_example[CLI example]
    examples --> rust_example[Rust API example]
    examples --> config_example[Config-driven example]

    cli_example --> cli_entry[bijux-dag entrypoints]
    rust_example --> api_entry[dag-core and dag-runtime crate exports]
    config_example --> runtime_path[configured runtime path]
```

## CLI Entrypoints

```bash
bijux-dag validate ./examples/simple.dag.json
bijux-dag run ./examples/simple.dag.json --out ./runs
bijux-dag explain ./runs/run-123
bijux-dag runs inspect run-123 --root ./runs
bijux-dag replay ./runs/run-123 --out ./runs/replay-123
bijux-dag diff ./runs/run-122 ./runs/run-123 --mode semantic --explain
```

## Rust Entrypoint Example

```rust
use bijux_dag_core::parse_graph_strict;

let graph = parse_graph_strict("{\"spec\":\"bijux-dag/v0.1\",\"nodes\":[],\"edges\":[]}")?;
println!("spec={}", graph.spec);
```

## Code Anchors

- `crates/bijux-dag-cli/src/main.rs`
- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-runtime/src/lib.rs`

## Next Reads

- [CLI Surface](cli-surface.md)
- [Operator Workflows](operator-workflows.md)
- [Local Development](../operations/local-development.md)
