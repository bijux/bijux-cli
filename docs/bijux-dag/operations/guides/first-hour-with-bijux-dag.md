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
its version and stable command surface, validate a small DAG, and run it with
visible artifacts.

For the first practical workflow after this bootstrap path, continue with
[File Processing Workflow](file-processing-workflow.md).

## First-hour sequence

1. Build the CLI with `cargo build -p bijux-dag-cli --release`.
2. Check the binary with `cargo run -p bijux-dag-cli --bin bijux-dag -- version`.
3. Inspect the stable support surface with `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`.
4. Validate `evidence/dag/authoring/examples/minimal_consumer.dag.json`.
5. Run the same fixture into a local `artifacts/` output root.
6. Inspect the resulting run with `bijux-dag explain` and `bijux-dag verify`.

## Concrete Walkthrough

```bash
cargo build -p bijux-dag-cli --release
cargo run -p bijux-dag-cli --bin bijux-dag -- version
cargo run -p bijux-dag-cli --bin bijux-dag -- commands
cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/dag/authoring/examples/minimal_consumer.dag.json
cargo run -p bijux-dag-cli --bin bijux-dag -- run \
  evidence/dag/authoring/examples/minimal_consumer.dag.json \
  --out artifacts/runs
```

After the run completes, use the reported run directory or run id to inspect
the evidence:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- explain artifacts/runs/<run-dir>
cargo run -p bijux-dag-cli --bin bijux-dag -- verify artifacts/runs/<run-dir>
```

You have a successful first hour when all of the following are true:

- `version` reports the expected build identity
- `commands` shows the stable root command inventory
- `validate` accepts the example DAG without semantic errors
- `run` creates a run directory under `artifacts/runs`
- `explain` and `verify` can read that run and report evidence instead of path
  or schema failures

## Boundary reminder

This walkthrough is intentionally local. It does not require container-cluster
deployment, external scheduler integration, or promoted remote coordination.
Maintainer-only probes such as `capabilities` remain outside this first-hour
operator contract and require `BIJUX_DAG_ENABLE_INTERNAL=1`.

## Next reads

- [File Processing Workflow](file-processing-workflow.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Installation And Setup](installation-and-setup.md)
- [Trust Boundaries](../reference/trust-boundaries.md)
- [Support Matrix](../../interfaces/reference/support-matrix.md)
