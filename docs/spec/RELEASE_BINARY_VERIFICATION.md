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

## Required Command Surface

```bash
bijux --json doctor
bijux --json cli paths
bijux dag version --json
bijux dag capabilities --json
bijux dag validate --json evidence/authoring/examples/hello.dag.json
bijux dag run --json evidence/authoring/examples/hello.dag.json --out ${RUN_ROOT}
bijux dag validate --json evidence/authoring/examples/etl-constant-to-shell.dag.json
bijux dag run --json evidence/authoring/examples/etl-constant-to-shell.dag.json --out ${RUN_ROOT}
bijux dag status --json ${RUN_DIR}
```

## Scenario Source of Truth

`configs/dag/release/release_smoke_scenarios.json` is the release scenario
contract consumed by the distribution delivery verification gate.
