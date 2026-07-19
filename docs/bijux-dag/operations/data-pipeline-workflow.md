---
title: Data Pipeline Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# Data Pipeline Workflow

This guide demonstrates a structured local data pipeline that ingests regional
sales data, cleans it, derives summary tables, validates totals, and publishes
a final revenue attainment table.

The workflow is backed by the repository example
`evidence/dag/authoring/examples/regional-sales-pipeline.dag.json` and the
sample inputs under `evidence/dag/authoring/examples/regional-sales-source`.

## What This Workflow Proves

- path-bound graph inputs drive a real structured data workflow
- a second run reuses cache across the full retained pipeline
- changing the orders input path invalidates the dependent stages
- an independent targets branch stays cached across the changed run
- `runs compare` identifies the changed stages and final artifact

## Prepare Inputs

Run these commands from repository root:

```bash
GRAPH_PATH="evidence/dag/authoring/examples/regional-sales-pipeline.dag.json"
ORDERS_CSV="$(pwd)/evidence/dag/authoring/examples/regional-sales-source/orders.csv"
TARGETS_JSON="$(pwd)/evidence/dag/authoring/examples/regional-sales-source/targets.json"
RUN_ROOT="./artifacts/regional-sales-runs"
CACHE_ROOT="./artifacts/regional-sales-cache"
UPDATED_ORDERS_CSV="./artifacts/regional-sales-inputs/orders-updated.csv"
mkdir -p ./artifacts/regional-sales-inputs
```

The workflow requires two runtime path inputs:

- `orders_csv` for the sales feed
- `targets_json` for the regional target ledger

## Validate The Graph

```bash
bijux-dag validate "${GRAPH_PATH}"
```

## Run The Cold And Warm Pipeline

Run the workflow once to populate cache and retain evidence:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id regional-sales-cold \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "orders_csv=${ORDERS_CSV}" \
  --input "targets_json=${TARGETS_JSON}" \
  --input "report_title=Regional Revenue Attainment"
```

Run it again with the same inputs:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id regional-sales-warm \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "orders_csv=${ORDERS_CSV}" \
  --input "targets_json=${TARGETS_JSON}" \
  --input "report_title=Regional Revenue Attainment"
```

On the warm run, the retained stages should be reused from cache:

1. `ingest_orders`
2. `clean_orders`
3. `derive_region_totals`
4. `derive_segment_totals`
5. `load_targets`
6. `validate_outputs`
7. `publish_final_table`

## Inspect The Final Table

The final table is written under the retained run:

```text
artifacts/regional-sales-runs/run-regional-sales-cold/nodes/publish_final_table/outputs/final/revenue_attainment.csv
```

Inspect the run and retained artifacts:

```bash
bijux-dag explain "${RUN_ROOT}/run-regional-sales-cold"
bijux-dag artifact registry "${RUN_ROOT}/run-regional-sales-cold" --json
```

The published table reports revenue, target, variance, and status per region.

## Change The Orders Input

Create an updated orders file at a new path so the changed graph input is
explicit in retained run evidence:

```bash
python3 - <<'PY'
from pathlib import Path

source = Path("evidence/dag/authoring/examples/regional-sales-source/orders.csv")
updated = Path("artifacts/regional-sales-inputs/orders-updated.csv")
updated.write_text(
    source.read_text().replace(
        "A-102,north,mid-market,3,15.00",
        "A-102,north,mid-market,5,15.00",
    ),
    encoding="utf-8",
)
PY
```

Run the workflow with the updated orders path while keeping the targets path
unchanged:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id regional-sales-updated \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "orders_csv=${UPDATED_ORDERS_CSV}" \
  --input "targets_json=${TARGETS_JSON}" \
  --input "report_title=Regional Revenue Attainment"
```

In this changed run, `load_targets` should remain cached while the
order-dependent stages rerun.

## Compare Retained Runs

Use retained-run comparison to attribute the change:

```bash
bijux-dag runs compare regional-sales-warm regional-sales-updated \
  --root "${RUN_ROOT}" \
  --json
```

The comparison should show:

- `input_values.changed_inputs` includes `orders_csv`
- `node_statuses.changed_nodes` includes the order-dependent stages
- `node_statuses.changed_nodes` does not include `load_targets`
- `output_hashes.changed_outputs` includes
  `publish_final_table:final/revenue_attainment.csv`

This is the practical proof that the workflow records which retained run
surfaces changed and which independent branch remained stable.

## Reading Rule

Use this guide when you need a realistic analytics-style DAG workflow that
demonstrates cache reuse, retained-run comparison, and selective invalidation
with explicit graph inputs.

## Next Reads

- [File Processing Workflow](file-processing-workflow.md)
- [Common Workflows](common-workflows.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
