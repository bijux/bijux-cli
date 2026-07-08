---
title: Cache Behavior Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Cache Behavior Workflow

This guide demonstrates the full cache behavior that `bijux-dag` currently
proves on one real repository workflow:

- the first run executes every retained stage
- the second run reuses cache across the full workflow
- changing one graph input invalidates only the dependent stages
- an unrelated branch stays cached
- a corrupted cache entry is refused instead of reused
- cache-miss explanation identifies both changed inputs and corruption refusal
- a corrupted retained run output can be restored from an exact verified cache
  entry through the internal repair lane

The workflow uses the repository regional sales example:

- graph: `evidence/dag/authoring/examples/regional-sales-pipeline.dag.json`
- inputs: `evidence/dag/authoring/examples/regional-sales-source/`

`cache verify` is part of the stable operator surface. `why-cache-missed` is
repository-tested and callable by explicit path, but it remains outside the
stable `bijux-dag --help` contract in `v0.4.0`.

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

## Validate The Graph

```bash
bijux-dag validate "${GRAPH_PATH}"
```

## Run The Cold And Warm Workflow

Populate cache and retain the first run:

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

Run the same workflow again with the same cache directory and inputs:

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

The cold run should execute all seven retained stages. The warm run should
reuse all seven from cache:

1. `ingest_orders`
2. `clean_orders`
3. `derive_region_totals`
4. `derive_segment_totals`
5. `load_targets`
6. `validate_outputs`
7. `publish_final_table`

The published table remains under retained evidence:

```text
artifacts/regional-sales-runs/run-regional-sales-warm/nodes/publish_final_table/outputs/final/revenue_attainment.csv
```

## Change One Graph Input

Write a modified orders file at a new path so the retained run records the new
input binding explicitly:

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

Run the workflow with the updated orders path while keeping the targets input
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

In this updated run:

- `load_targets` should stay cached
- the order-dependent stages should rerun

Use retained-run comparison to confirm the changed surface:

```bash
bijux-dag runs compare regional-sales-warm regional-sales-updated \
  --root "${RUN_ROOT}" \
  --json
```

The comparison should show:

- `input_values.changed_inputs` includes `orders_csv`
- `node_statuses.changed_nodes` includes the order-dependent stages
- `node_statuses.changed_nodes` does not include `load_targets`

## Explain The Changed-Input Cache Miss

Use the explicit-path explanation route against the updated run:

```bash
bijux-dag --json why-cache-missed \
  --run-dir "${RUN_ROOT}/run-regional-sales-updated" \
  --node clean_orders \
  --cache-dir "${CACHE_ROOT}"
```

The response should report:

- `data.mode = "node"`
- `data.outcome = "miss"`
- `data.taxonomy` includes `changed_input_hashes`

This is the direct explanation for why the changed orders path invalidated the
`clean_orders` cache candidate.

## Corrupt One Real Cache Entry

Read the exact cache key used by `load_targets` on the warm run:

```bash
export LOAD_TARGETS_CACHE_KEY="$(
  python3 - <<'PY'
import json
from pathlib import Path

trace = Path("artifacts/regional-sales-runs/run-regional-sales-warm/nodes/load_targets/trace.json")
payload = json.loads(trace.read_text())
print(payload["cache_identity"]["cache_key"])
PY
)"
```

Tamper with one retained payload under that cache entry:

```bash
python3 - <<'PY'
from pathlib import Path
import os

entry = Path("artifacts/regional-sales-cache") / os.environ["LOAD_TARGETS_CACHE_KEY"] / "outputs"
for candidate in entry.rglob("*"):
    if candidate.is_file() and candidate.name != "index.json":
        candidate.write_text("corrupted-cache-payload\n", encoding="utf-8")
        print(candidate)
        break
else:
    raise SystemExit(f"no payload found under {entry}")
PY
```

## Verify Cache Integrity

Run cache verification against the same cache directory:

```bash
bijux-dag --json cache verify --cache-dir "${CACHE_ROOT}"
```

The command should exit with status `3` and report at least one corrupt entry
under `data.corrupt_total`.

That is the integrity boundary working as intended: the entry still exists, but
the runtime no longer trusts it.

## Explain The Corruption Refusal

Ask for the cache explanation on the original warm-run node that previously hit
cache:

```bash
bijux-dag --json why-cache-missed \
  --run-dir "${RUN_ROOT}/run-regional-sales-warm" \
  --node load_targets \
  --cache-dir "${CACHE_ROOT}"
```

The response should now report:

- `data.outcome = "unsafe_reuse_refused"`
- `data.exact_entry_report.eligible = false`
- `data.taxonomy` is populated with corruption-related reasons

This is the operator-facing proof that cache reuse is rejected when integrity
verification fails.

## Restore A Corrupted Retained Output From Cache

Corrupt one retained output in the warm run while leaving the cache entry
itself untouched:

```bash
python3 - <<'PY'
from pathlib import Path

target = Path(
    "artifacts/regional-sales-runs/run-regional-sales-warm/"
    "nodes/publish_final_table/outputs/final/revenue_attainment.csv"
)
target.write_text("corrupted-retained-output\n", encoding="utf-8")
print(target)
PY
```

The stable integrity surface should now fail on that retained run:

```bash
bijux-dag --json verify "${RUN_ROOT}/run-regional-sales-warm"
```

Restore the retained output from the exact verified cache entry:

```bash
BIJUX_DAG_ENABLE_INTERNAL=1 bijux-dag --json runtime repair --apply \
  --cache-dir "${CACHE_ROOT}" \
  --out "${RUN_ROOT}" \
  "${RUN_ROOT}/run-regional-sales-warm"
```

The repair report should show:

- `data.cache_recoveries_applied[0].node_id = "publish_final_table"`
- `data.repair_run = null`
- `data.issues = []`

That is the honest boundary of this repair lane: it restores retained outputs
only when the exact persisted cache entry still verifies. If the cache entry is
also missing or corrupt, repair must fall back to a child rerun or remain
failed.

## Rerun After Corruption

Run the original workflow again with the original inputs:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id regional-sales-corrupt \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "orders_csv=${ORDERS_CSV}" \
  --input "targets_json=${TARGETS_JSON}" \
  --input "report_title=Regional Revenue Attainment"
```

In this rerun:

- `load_targets` should execute again because its exact cache entry was refused
- the order branch should remain cached because its cache entries stayed valid
- the final revenue table should match the warm run output

Inspect the retained traces directly if you need the per-node status proof:

```text
artifacts/regional-sales-runs/run-regional-sales-corrupt/nodes/load_targets/trace.json
artifacts/regional-sales-runs/run-regional-sales-corrupt/nodes/clean_orders/trace.json
artifacts/regional-sales-runs/run-regional-sales-corrupt/nodes/derive_region_totals/trace.json
artifacts/regional-sales-runs/run-regional-sales-corrupt/nodes/derive_segment_totals/trace.json
```

## Reading Rule

Use this guide when the question is whether `bijux-dag` cache reuse is both
selective and integrity-checked on a real retained workflow, not only on
synthetic fixtures.

## Next Reads

- [Data Pipeline Workflow](data-pipeline-workflow.md)
- [Common Workflows](../common-workflows.md)
- [Operator Workflows](../../interfaces/operator-workflows.md)
