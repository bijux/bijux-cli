---
title: Scheduled Catalog Refresh Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# Scheduled Catalog Refresh Workflow

This guide demonstrates a real scheduled workflow built around
`evidence/dag/authoring/examples/scheduled-catalog-refresh.dag.json`.

It exercises the current maintained internal `schedule` lane: cron preview,
submission ledger creation, queue dispatch, explicit handoff into a real DAG
run, and queue completion after that run finishes.

This is not presented as a stable public scheduler service in `v0.4.0`.
These commands remain repository-tested internal routes and require
`BIJUX_DAG_ENABLE_INTERNAL=1`.

## What This Workflow Proves

- a cron schedule computes the next fire time deterministically
- the emitted schedule timestamp becomes a real DAG graph input
- re-evaluating the same cron slot does not create a second submission
- the scheduled run id remains visible from queue ledger to run manifest

## Prepare The Run

Run these commands from repository root:

```bash
export BIJUX_DAG_ENABLE_INTERNAL=1

GRAPH_PATH="evidence/dag/authoring/examples/scheduled-catalog-refresh.dag.json"
RUN_ROOT="./artifacts/scheduled-catalog-refresh-runs"
SCHEDULE_REGISTRY="./artifacts/scheduled-catalog-refresh-registry.json"
SCHEDULE_INPUTS="./artifacts/scheduled-catalog-refresh-inputs.json"
SCHEDULE_LEDGER="./artifacts/scheduled-catalog-refresh-ledger.json"
RUN_INPUTS="./artifacts/scheduled-catalog-refresh-run-inputs.json"
STATUS_UPDATES="./artifacts/scheduled-catalog-refresh-status-updates.json"
```

Write the schedule registry:

```bash
cat > "${SCHEDULE_REGISTRY}" <<'EOF'
{
  "definitions": [
    {
      "id": "catalog-refresh-hourly",
      "dag_name": "atlas.catalog-refresh",
      "dag_version_policy": "run-latest",
      "input_contract": {
        "scheduled_at_unix_ms": { "type": "integer", "required": true },
        "refresh_label": { "type": "string", "default": "Nightly Catalog Refresh" },
        "dataset_name": { "type": "string", "default": "atlas.catalog" }
      },
      "input_bindings": {
        "scheduled_at_unix_ms": { "source": "requested_unix_ms" }
      },
      "trigger": {
        "Cron": {
          "expression": "0 * * * *",
          "timezone": "UTC"
        }
      },
      "queue": { "queue_name": "catalog-refresh", "tenant": "atlas" },
      "priority": "High",
      "concurrency": {
        "per_dag": 1,
        "per_queue": 2,
        "per_tenant": 2,
        "per_node_group": null
      },
      "catch_up": { "enabled": false, "max_catch_up_runs": 0 }
    }
  ]
}
EOF
```

Validate the graph and the schedule registry:

```bash
bijux-dag validate "${GRAPH_PATH}"
bijux-dag --json schedule validate "${SCHEDULE_REGISTRY}"
```

## Preview The Next Cron Slot

Preview the schedule shortly before the target hour:

```bash
bijux-dag --json schedule preview "${SCHEDULE_REGISTRY}" \
  --now-unix-ms 1768473000000 \
  --next-runs 1
```

The preview should report `next_fire_unix_ms = 1768474800000`, which is
`2026-01-15T11:00:00Z`.

## Emit The Submission Ledger

Write the schedule evaluation input for that exact cron slot:

```bash
cat > "${SCHEDULE_INPUTS}" <<'EOF'
{
  "now_unix_ms": 1768474800000
}
EOF
```

Evaluate the schedule into the durable submission ledger:

```bash
bijux-dag --json schedule submit "${SCHEDULE_REGISTRY}" \
  "${SCHEDULE_INPUTS}" \
  --out "${SCHEDULE_LEDGER}"
```

At this point the ledger should contain one pending scheduled run with:

- `scheduled_at_unix_ms = 1768474800000`
- `refresh_label = "Nightly Catalog Refresh"`
- `dataset_name = "atlas.catalog"`
- one deterministic `run_id`

## Prove The Same Slot Is Not Submitted Twice

Re-evaluate the same slot against the existing ledger:

```bash
bijux-dag --json schedule submit "${SCHEDULE_REGISTRY}" \
  "${SCHEDULE_INPUTS}" \
  --ledger "${SCHEDULE_LEDGER}" \
  --out "${SCHEDULE_LEDGER}"
```

For this cron-triggered workflow, the second evaluation emits no new request
and leaves the ledger at one recorded submission for that slot.

## Dispatch The Queued Run

Inspect queue occupancy:

```bash
bijux-dag --json schedule queue status "${SCHEDULE_REGISTRY}" \
  --ledger "${SCHEDULE_LEDGER}"
```

Dispatch the pending submission:

```bash
bijux-dag --json schedule queue dispatch "${SCHEDULE_LEDGER}" \
  --max-dispatches 1 \
  --out "${SCHEDULE_LEDGER}"
```

Extract the scheduled run id and the exact graph inputs from the ledger:

```bash
SCHEDULED_RUN_ID="$(python3 - <<'PY' "${SCHEDULE_LEDGER}"
import json
import sys

entry = json.load(open(sys.argv[1]))["entries"][0]
print(entry["run_id"])
PY
)"

python3 - <<'PY' "${SCHEDULE_LEDGER}" "${RUN_INPUTS}"
import json
import sys

entry = json.load(open(sys.argv[1]))["entries"][0]
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    json.dump(entry["graph_inputs"], handle, indent=2)
    handle.write("\n")
PY
```

## Execute The Real DAG Run

Run the repository workflow with the dispatched run id and the ledger-owned
graph inputs:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id "${SCHEDULED_RUN_ID}" \
  --inputs-file "${RUN_INPUTS}"
```

Inspect the retained publication and manifest:

```bash
cat "${RUN_ROOT}/run-${SCHEDULED_RUN_ID}/nodes/render_refresh_report/outputs/publish/report.md"
cat "${RUN_ROOT}/run-${SCHEDULED_RUN_ID}/manifest.json"
bijux-dag verify --json "${RUN_ROOT}/run-${SCHEDULED_RUN_ID}" --strict
```

The retained evidence should show:

- the same `run_id` recorded in both the schedule ledger and the run manifest
- `run_metadata.graph_inputs.scheduled_at_unix_ms = 1768474800000`
- a report line `Scheduled at: 2026-01-15T11:00:00Z`

This is the honest current handoff: the schedule lane prepares and tracks the
submission, and the DAG execution still runs explicitly with the emitted run
id and graph inputs.

## Close The Queue Entry

Record the finished run back into the schedule ledger:

```bash
cat > "${STATUS_UPDATES}" <<'EOF'
{
  "updates": [
    {
      "run_id": "__RUN_ID__",
      "status": "Completed",
      "updated_unix_ms": 1768475100000
    }
  ]
}
EOF

python3 - <<'PY' "${STATUS_UPDATES}" "${SCHEDULED_RUN_ID}"
import json
import sys

payload = json.load(open(sys.argv[1]))
payload["updates"][0]["run_id"] = sys.argv[2]
with open(sys.argv[1], "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
    handle.write("\n")
PY

bijux-dag --json schedule queue update "${SCHEDULE_LEDGER}" \
  "${STATUS_UPDATES}" \
  --out "${SCHEDULE_LEDGER}"

bijux-dag --json schedule queue status "${SCHEDULE_REGISTRY}" \
  --ledger "${SCHEDULE_LEDGER}"
```

The queue should now report zero active runs.

## Reading Rule

Use this guide when the question is not whether scheduler primitives exist,
but whether the current repository-backed schedule lane can preview one cron
slot, emit one durable submission, hand that submission into a real run, and
close the queue state without hidden state.

## Next Reads

- [Operator Workflows](../../interfaces/operator-workflows.md)
- [CLI Surface](../../interfaces/cli-surface.md)
- [Common Workflows](../common-workflows.md)
- [Compliance-Gated Bulletin Workflow](compliance-gated-bulletin-workflow.md)
