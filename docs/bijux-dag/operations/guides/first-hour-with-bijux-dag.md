---
title: First Hour With Bijux Dag
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# First Hour With Bijux Dag

The first hour should prove that a local operator can build the CLI, inspect
its version and stable command surface, and then move into one retained
workflow that demonstrates cache, replay, and artifact evidence.

Use this page for the bootstrap half of that path. For the first complete
retained workflow after bootstrap, continue with
[First-Run Tutorial](first-run-tutorial.md).

If you do not need the bootstrap walkthrough and only want one command that
proves the retained local DAG surface, run `make dag-demo` from repository
root. The rest of this page explains the slower path that builds confidence in
the command surface before that retained workflow.

## First-hour sequence

1. Build the CLI with `cargo build -p bijux-dag-cli --release`.
2. Check the binary with `cargo run -p bijux-dag-cli --bin bijux-dag -- version`.
3. Inspect the stable support surface with `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`.
4. Validate `evidence/dag/authoring/examples/minimal_consumer.dag.json`.
5. Run the same fixture into a local `artifacts/` output root.
6. Inspect the resulting run with `bijux-dag explain` and `bijux-dag verify`.
7. Continue with the retained file-processing tutorial for cache, artifacts,
   and replay.

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

You have completed the bootstrap hour when all of the following are true:

- `version` reports the expected build identity
- `commands` shows the stable root command inventory
- `validate` accepts the example DAG without semantic errors
- `run` creates a run directory under `artifacts/runs`
- `explain` and `verify` can read that run and report evidence instead of path
  or schema failures
- you are ready to continue into [First-Run Tutorial](first-run-tutorial.md)
  for the first retained workflow family

## Boundary reminder

This walkthrough is intentionally local. It does not require container-cluster
deployment, external scheduler integration, or promoted remote coordination.
Maintainer-only probes such as `capabilities` remain outside this first-hour
operator contract and require `BIJUX_DAG_ENABLE_INTERNAL=1`.

If the next question is whether a flag or backend is a real isolation boundary,
open [Security And Isolation Truth](../reference/security-isolation-truth.md)
before assuming shell policy flags behave like a sandbox.

For the full `v0.4.0` release-boundary classification behind that distinction,
use
[`contracts/foundation/dag_release_truth_table.v1.json`](../../../../contracts/foundation/dag_release_truth_table.v1.json)
and
[`docs/bijux-dag/foundation/release-boundary.md`](../../foundation/release-boundary.md).

## Next reads

- [First-Run Tutorial](first-run-tutorial.md)
- [File Processing Workflow](file-processing-workflow.md)
- [Data Pipeline Workflow](data-pipeline-workflow.md)
- [Operator Workflows](../../interfaces/operator-workflows.md)
- [Installation And Setup](../installation-and-setup.md)
- [Security And Isolation Truth](../reference/security-isolation-truth.md)
- [Trust Boundaries](../reference/trust-boundaries.md)
- [Support Matrix](../../interfaces/reference/support-matrix.md)
