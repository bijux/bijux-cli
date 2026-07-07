---
title: Historical Catalog Backfill Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# Historical Catalog Backfill Workflow

This guide demonstrates a real repository-backed backfill workflow built around
`evidence/dag/authoring/examples/historical-catalog-backfill.dag.json`.

It exercises the current maintained internal `schedule backfill` lane:
deterministic partition fanout, aggregate summary reporting, failed-partition
retry, and explicit handoff into retained DAG runs.

This is not presented as a stable public scheduler service in `v0.4.0`.
These commands remain repository-tested internal routes and require
`BIJUX_DAG_ENABLE_INTERNAL=1`.

## What This Workflow Proves

- a bounded three-minute backfill window expands into one run per partition per
  minute slot
- the emitted run requests carry both the overall backfill window and the
  specific `requested_unix_ms` slot into graph inputs
- a failed partition can be re-queued with a new run id while preserving
  attempt history
- an aggregate backfill summary reports queued, submitted, failed, and retried
  counts from the durable state file

## Prepare The Run

Run these commands from repository root:

```bash
export BIJUX_DAG_ENABLE_INTERNAL=1

GRAPH_PATH="evidence/dag/authoring/examples/historical-catalog-backfill.dag.json"
RUN_ROOT="./artifacts/historical-catalog-backfill-runs"
BACKFILL_REGISTRY="./artifacts/historical-catalog-backfill-registry.json"
BACKFILL_STATE="./artifacts/historical-catalog-backfill-state.json"
BACKFILL_SUMMARY="./artifacts/historical-catalog-backfill-summary.json"
BACKFILL_ADVANCE_REQUEST="./artifacts/historical-catalog-backfill-advance-request.json"
BACKFILL_ADVANCE_REPORT="./artifacts/historical-catalog-backfill-advance-report.json"
FIRST_RUN_INPUTS="./artifacts/historical-catalog-backfill-first-run-inputs.json"
FAILED_REQUEST="./artifacts/historical-catalog-backfill-failed-request.json"
FAILED_STATE="./artifacts/historical-catalog-backfill-failed-state.json"
RETRIED_STATE="./artifacts/historical-catalog-backfill-retried-state.json"
RETRIED_SUMMARY="./artifacts/historical-catalog-backfill-retried-summary.json"
BACKFILL_RETRY_REPORT="./artifacts/historical-catalog-backfill-retry-report.json"
RETRY_RUN_INPUTS="./artifacts/historical-catalog-backfill-retry-run-inputs.json"
```

Write the backfill registry:

```bash
cat > "${BACKFILL_REGISTRY}" <<'EOF'
{
  "definitions": [
    {
      "id": "catalog-history",
      "dag_name": "atlas.catalog-backfill",
      "dag_version_policy": "run-latest",
      "input_contract": {
        "requested_unix_ms": { "type": "integer", "required": true },
        "backfill_window_start_unix_ms": { "type": "integer", "required": true },
        "backfill_window_end_unix_ms": { "type": "integer", "required": true },
        "backfill_partition_key": { "type": "string", "required": true },
        "catalog_name": { "type": "string", "default": "atlas.catalog" },
        "publication_title": { "type": "string", "default": "Historical Catalog Backfill" }
      },
      "input_bindings": {
        "requested_unix_ms": { "source": "requested_unix_ms" },
        "backfill_window_start_unix_ms": { "source": "backfill_window_start_unix_ms" },
        "backfill_window_end_unix_ms": { "source": "backfill_window_end_unix_ms" },
        "backfill_partition_key": { "source": "backfill_partition_key" }
      },
      "trigger": {
        "Backfill": {
          "window_start_unix_ms": 1704067200000,
          "window_end_unix_ms": 1704067320000,
          "partition_by": "region",
          "partition_keys": ["north-america", "europe"],
          "max_parallelism": 2,
          "failure_policy": "pause"
        }
      },
      "queue": { "queue_name": "catalog-backfill", "tenant": "atlas" },
      "priority": "High",
      "concurrency": {
        "per_dag": 2,
        "per_queue": 4,
        "per_tenant": 4,
        "per_node_group": null
      },
      "catch_up": { "enabled": false, "max_catch_up_runs": 0 }
    }
  ]
}
EOF
```

Write the advance request used for dispatch:

```bash
cat > "${BACKFILL_ADVANCE_REQUEST}" <<'EOF'
{
  "now_unix_ms": 1704067105000,
  "pending_live_runs": 0,
  "throttling_policy": {
    "max_backfill_submissions_per_tick": 6,
    "reserve_live_capacity_percent": 0
  },
  "status_updates": []
}
EOF
```

Validate the graph and the backfill registry:

```bash
bijux-dag validate "${GRAPH_PATH}"
bijux-dag --json schedule validate "${BACKFILL_REGISTRY}"
```

## Plan The Backfill

Plan the operation into a durable state file:

```bash
bijux-dag --json schedule backfill plan "${BACKFILL_REGISTRY}" \
  --schedule-id catalog-history \
  --planned-unix-ms 1704067100000 \
  --backfill-id catalog-history-january \
  --out "${BACKFILL_STATE}"
```

Summarize the planned state:

```bash
bijux-dag --json schedule backfill summary "${BACKFILL_STATE}" \
  --out "${BACKFILL_SUMMARY}"
```

At this point the durable state should report:

- `total_runs = 6`
- `queued_runs = 6`
- one-minute requested slots at `2024-01-01T00:00:00Z`,
  `2024-01-01T00:01:00Z`, and `2024-01-01T00:02:00Z`
- two partitions, `north-america` and `europe`, for every slot

The overall backfill window stays fixed from `2024-01-01T00:00:00Z` through
`2024-01-01T00:02:00Z`, while `requested_unix_ms` identifies the specific
minute slot for each emitted run.

## Dispatch The First Partition Runs

Advance the operation and capture the JSON report:

```bash
bijux-dag --json schedule backfill advance \
  "${BACKFILL_STATE}" \
  "${BACKFILL_ADVANCE_REQUEST}" \
  --out "${BACKFILL_STATE}" \
  > "${BACKFILL_ADVANCE_REPORT}"
```

Extract the first runnable partition and the partition that will be marked as
failed:

```bash
FIRST_RUN_ID="$(python3 - <<'PY' "${BACKFILL_ADVANCE_REPORT}" "${FIRST_RUN_INPUTS}"
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))["data"]["dispatched_requests"]
first = payload[0]
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    json.dump(first["graph_inputs"], handle, indent=2)
    handle.write("\n")
print(first["run_id"])
PY
)"

FAILED_RUN_ID="$(python3 - <<'PY' "${BACKFILL_ADVANCE_REPORT}"
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))["data"]["dispatched_requests"]
print(payload[1]["run_id"])
PY
)"
```

The first emitted request should bind:

- `requested_unix_ms = 1704067200000`
- `backfill_window_start_unix_ms = 1704067200000`
- `backfill_window_end_unix_ms = 1704067320000`
- `backfill_partition_key = "north-america"`

Run that first partition with the emitted run id and graph inputs:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id "${FIRST_RUN_ID}" \
  --inputs-file "${FIRST_RUN_INPUTS}"
```

Inspect and verify the retained run:

```bash
cat "${RUN_ROOT}/run-${FIRST_RUN_ID}/nodes/render_partition_report/outputs/publish/report.md"
bijux-dag verify --json "${RUN_ROOT}/run-${FIRST_RUN_ID}" --strict
```

The retained report should include:

- `Partition: north-america`
- `Requested slot: 2024-01-01T00:00:00Z`
- `Window start: 2024-01-01T00:00:00Z`
- `Window end: 2024-01-01T00:02:00Z`

## Mark A Partition As Failed

Record a failed status update for the second emitted partition:

```bash
cat > "${FAILED_REQUEST}" <<EOF
{
  "now_unix_ms": 1704067110000,
  "pending_live_runs": 0,
  "throttling_policy": {
    "max_backfill_submissions_per_tick": 6,
    "reserve_live_capacity_percent": 0
  },
  "status_updates": [
    {
      "run_id": "${FAILED_RUN_ID}",
      "status": "failed",
      "updated_unix_ms": 1704067109000
    }
  ]
}
EOF
```

Advance the durable state with that failure:

```bash
bijux-dag --json schedule backfill advance \
  "${BACKFILL_STATE}" \
  "${FAILED_REQUEST}" \
  --out "${FAILED_STATE}"
```

With `failure_policy = "pause"`, the operation should now report
`lifecycle = "paused"` and should not dispatch additional queued runs.

## Retry The Failed Partition

Re-queue only the failed partition:

```bash
bijux-dag --json schedule backfill retry-failed "${FAILED_STATE}" \
  --at-unix-ms 1704067115000 \
  --out "${RETRIED_STATE}"
```

Summarize the retried state:

```bash
bijux-dag --json schedule backfill summary "${RETRIED_STATE}" \
  --out "${RETRIED_SUMMARY}"
```

The retried summary should now report:

- `submitted_runs = 1`
- `queued_runs = 5`
- `failed_runs = 0`
- `total_retry_attempts = 1`

That shape is intentional and honest: one earlier partition is still recorded
as submitted until its completion is written back, while the failed partition
has been re-queued with a new attempt and a preserved `previous_run_ids`
history.

Advance the retried state again and capture the emitted retry request:

```bash
bijux-dag --json schedule backfill advance \
  "${RETRIED_STATE}" \
  "${BACKFILL_ADVANCE_REQUEST}" \
  --out "${RETRIED_STATE}" \
  > "${BACKFILL_RETRY_REPORT}"
```

Extract the retry run id and retry graph inputs:

```bash
RETRY_RUN_ID="$(python3 - <<'PY' "${BACKFILL_RETRY_REPORT}" "${RETRY_RUN_INPUTS}"
import json
import sys

payload = json.load(open(sys.argv[1], encoding="utf-8"))["data"]["dispatched_requests"][0]
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    json.dump(payload["graph_inputs"], handle, indent=2)
    handle.write("\n")
print(payload["run_id"])
PY
)"
```

Run and verify the retried partition:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id "${RETRY_RUN_ID}" \
  --inputs-file "${RETRY_RUN_INPUTS}"

cat "${RUN_ROOT}/run-${RETRY_RUN_ID}/nodes/render_partition_report/outputs/publish/report.md"
bijux-dag verify --json "${RUN_ROOT}/run-${RETRY_RUN_ID}" --strict
```

The retried run should now prove:

- the partition is `europe`
- the requested slot remains `2024-01-01T00:00:00Z`
- the retried run id differs from the failed run id
- the durable backfill state retains the previous failed run id in
  `previous_run_ids`

This is the honest current handoff for backfill work in `v0.4.0`: the
internal backfill lane expands, tracks, summarizes, and retries durable run
requests, while DAG execution still runs explicitly with the emitted run ids
and graph inputs.
