---
title: CI Integration
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# Ci Integration

CI integration for `bijux-dag` should prove the visible DAG surface in a clean
environment before any repository-specific automation layers are added.

## Minimum lane

```bash
cargo build -p bijux-dag-cli --release
cargo run -p bijux-dag-cli --bin bijux-dag -- version
cargo run -p bijux-dag-cli --bin bijux-dag -- commands
```

Maintainer-only CI probes may add
`BIJUX_DAG_ENABLE_INTERNAL=1 cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json`,
but that probe is not part of the public operator boundary.

The contract source for that release distinction is
[`contracts/foundation/dag_release_truth_table.v1.json`](../../../../contracts/foundation/dag_release_truth_table.v1.json)
and the handbook page
[`docs/bijux-dag/foundation/release-boundary.md`](../../foundation/release-boundary.md).

## Fixture bootstrap

- use `evidence/dag/authoring/examples/minimal_consumer.dag.json` as the
  minimum validation fixture
- keep run outputs under a repository-owned `artifacts/` root
- treat non-zero exits from `validate`, `run`, `verify`, or `doctor` as CI
  failures

## Next reads

- [Installation And Setup](../installation-and-setup.md)
- [First Hour With Bijux Dag](first-hour-with-bijux-dag.md)
