---
title: First Hour With Bijux Dag
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# First Hour With Bijux Dag

The first hour should prove that a local operator can build the CLI, inspect
its version and stable command surface, validate a small DAG, and run it with visible
artifacts.

## First-hour sequence

1. Build the CLI with `cargo build -p bijux-dag-cli --release`.
2. Check the binary with `cargo run -p bijux-dag-cli --bin bijux-dag -- version`.
3. Inspect the stable support surface with `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`.
4. Validate `evidence/authoring/examples/minimal_consumer.dag.json`.
5. Run the same fixture into a local `artifacts/` output root.
6. Inspect the resulting run with `bijux-dag explain` and `bijux-dag verify`.

## Boundary reminder

This walkthrough is intentionally local. It does not require container-cluster
deployment, external scheduler integration, or promoted remote coordination.
Maintainer-only probes such as `capabilities` remain outside this first-hour
operator contract and require `BIJUX_DAG_ENABLE_INTERNAL=1`.

## Next reads

- [Installation And Setup](installation-and-setup.md)
- [Trust Boundaries](trust-boundaries.md)
- [Support Matrix](../interfaces/support-matrix.md)
