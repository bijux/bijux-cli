---
title: Release Binary Verification
audience: mixed
type: reference
status: canonical
owner: bijux-dev
last_reviewed: 2026-05-01
---

# Release Binary Verification

Release artifacts are only considered runnable when they prove two first-run
paths from a clean environment:

- hello DAG run
- shell ETL DAG run

The command surfaces below are the minimum release proof contract.

This is a maintainer release-gate recipe, not a public operator quickstart. It
therefore includes one internal probe (`capabilities`) in addition to the
stable operator commands from the `v0.4.0` release boundary. That maintainer
probe now requires `BIJUX_DAG_ENABLE_INTERNAL=1` so the release recipe matches
the executable boundary enforced by the binary.

The release-boundary source of truth for this distinction is
[`contracts/foundation/dag_release_truth_table.v1.json`](../../contracts/foundation/dag_release_truth_table.v1.json)
plus the handbook page
[`docs/bijux-dag/foundation/release-boundary.md`](../bijux-dag/foundation/release-boundary.md).

## Required Command Surface

```bash
bijux --json doctor
bijux --json cli paths
bijux-dag version --json
BIJUX_DAG_ENABLE_INTERNAL=1 bijux-dag capabilities --json
bijux-dag validate --json evidence/dag/authoring/examples/hello.dag.json
bijux-dag run --json evidence/dag/authoring/examples/hello.dag.json --out ${RUN_ROOT}
bijux-dag validate --json evidence/dag/authoring/examples/etl-constant-to-shell.dag.json
bijux-dag run --json evidence/dag/authoring/examples/etl-constant-to-shell.dag.json --out ${RUN_ROOT}
bijux-dag explain --json ${RUN_DIR}
```

## Scenario Source of Truth

`configs/dag/release/release_smoke_scenarios.json` is the release scenario
contract consumed by the distribution delivery verification gate.
