---
title: CI Integration
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# CI Integration

Use this page when you need the minimum honest CI lane for `bijux-dag` without
blurring the public operator surface and the maintainer-only support surface.

For install prerequisites, open
[Installation And Setup](installation-and-setup.md). The
[Release Boundary](../foundation/release-boundary.md), backed by
`contracts/foundation/dag_release_truth_table.v1.json`, governs which commands
belong in an operator CI lane.

## Minimum lane

```bash
cargo build -p bijux-dag-cli --release
cargo run -p bijux-dag-cli --bin bijux-dag -- version
cargo run -p bijux-dag-cli --bin bijux-dag -- commands
```

Maintainer-only CI probes may add
`BIJUX_DAG_ENABLE_INTERNAL=1 cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json`,
but that probe is not part of the public operator boundary.

## Fixture bootstrap

- use `evidence/authoring/examples/minimal_consumer.dag.json` as the minimum
  validation fixture
- keep run outputs under a repository-owned `artifacts/` root
- treat non-zero exits from `validate`, `run`, `verify`, or `doctor` as CI
  failures
