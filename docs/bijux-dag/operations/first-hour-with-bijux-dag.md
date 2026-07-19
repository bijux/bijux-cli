---
title: First Hour With Bijux Dag
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# First Hour With Bijux Dag

Use this page when you need the shortest operator-facing bootstrap path from a
fresh checkout to one retained `bijux-dag` run.

Read [Security And Isolation Truth](security-isolation-truth.md) before treating
policy flags or an execution backend as a host sandbox.

For the longer tutorial sequence, open
[First Hour Guide](first-hour-with-bijux-dag.md) and
[First-Run Tutorial](first-run-tutorial.md).

## First-hour sequence

1. Build the CLI with `cargo build -p bijux-dag-cli --release`.
2. Check the binary with `cargo run -p bijux-dag-cli --bin bijux-dag -- version`.
3. Inspect the stable support surface with `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`.
4. Validate `evidence/authoring/examples/minimal_consumer.dag.json`.
5. Run the same fixture into a local `artifacts/` output root.
6. Inspect the resulting run with `bijux-dag explain` and `bijux-dag verify`.
7. Continue with the retained workflow tutorials for cache, artifacts, and replay.

## Concrete walkthrough

```bash
cargo build -p bijux-dag-cli --release
cargo run -p bijux-dag-cli --bin bijux-dag -- version
cargo run -p bijux-dag-cli --bin bijux-dag -- commands
cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/authoring/examples/minimal_consumer.dag.json
cargo run -p bijux-dag-cli --bin bijux-dag -- run \
  evidence/authoring/examples/minimal_consumer.dag.json \
  --out artifacts/runs
```

## Boundary reminder

Maintainer-only probes such as `capabilities` remain outside this first-hour
operator contract and require `BIJUX_DAG_ENABLE_INTERNAL=1`.
