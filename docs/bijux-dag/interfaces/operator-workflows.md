---
title: Operator Workflows
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Operator Workflows

This page explains the normal operator path through a DAG run when the goal is
to move from definition to evidence-backed judgment.

The important habit is simple: validate first, run once, explain evidence, then
use replay or diff instead of guessing.

## Workflow Map

```mermaid
flowchart LR
    define["validate definition"] --> run["execute run"]
    run --> inspect["explain run and inspect artifacts"]
    inspect --> replay["replay if reproducibility matters"]
    replay --> diff["diff if attribution matters"]
```

## Baseline Workflow

1. validate graph definition and canonical form
2. execute run and collect run id
3. explain run and inspect artifact evidence
4. replay for reproducibility classification
5. diff against baseline for scoped drift attribution

## Example Sequence

```bash
bijux-dag validate ./pipelines/main.dag.json
bijux-dag run ./pipelines/main.dag.json --out ./runs
bijux-dag explain ./runs/run-20260406-01
bijux-dag runs inspect run-20260406-01 --root ./runs
bijux-dag replay ./runs/run-20260406-01 --out ./runs/replay
bijux-dag diff ./runs/run-20260405-77 ./runs/run-20260406-01 --mode semantic --explain
```

After `bijux-dag run`, the command now prints a compact post-run summary in
human mode and exposes the same structure under `data.summary` in JSON mode.

That completion summary answers the immediate operator questions without forcing
an extra inspect call:

- final status
- elapsed `duration_ms`
- `node_counts`
- failed node reasons when the run did not succeed
- `cache_hits`
- `artifact_count`
- `promoted_artifact_count`
- one suggested next action with a concrete follow-up command

## Query Recent Run Status

When you need the current status lane instead of a single-run deep dive, use
`runs history` as the status surface for both active and historical runs.

```bash
bijux-dag runs history --root ./runs --status success --limit 5
bijux-dag runs history --json --root ./runs --graph training-pipeline --limit 10
```

The history rows now report:

- `status` and `lifecycle_state` so active runs are distinguishable from
  finalized historical ones
- `graph_name` and `graph_fingerprint` for first-class graph filtering
- `parent_run_id`, `source_run_id`, and `lineage.child_run_ids` for run
  ancestry
- `run_dir` and `output_location` so the evidence location is visible without
  an extra lookup

Use the JSON form when another tool needs stable fields, and use the default
human form when an operator needs a compact recent-run status list.

## Stop an Active Run

When a live run should stop launching new work, record a durable stop request
against its run id:

```bash
bijux-dag runs stop run-20260406-01 --root ./runs
bijux-dag runs stop run-20260406-01 --root ./runs --json
```

The stop request is written into the active run staging directory, the runtime
observes it during execution, and the finalized manifest records
`run_cancellation_cause = "operator_request"` when the stop succeeds.

## Preview Resolved Paths Before Execution

When a graph uses `{run_dir}`, `{work_dir}`, `{inputs_dir}`, `{outputs_dir}`, or
`{cache_dir}`, preview the concrete bindings before the first run instead of
guessing where the runtime will materialize them:

```bash
bijux-dag plan explain ./pipelines/main.dag.json \
  --json \
  --out ./runs \
  --run-id rehearsal-main \
  --cache-dir ./.bijux/cache
```

The JSON payload reports:

- `run_layout`: the previewed staging and final run directories
- `path_previews`: the resolved path expressions per node
- `execution_cost_estimate`: the selected node count, root set, critical path
  length, weighted `critical_path` details, topology-limited parallelism,
  resource demand, cache exposure, and timeout/retry exposure
- `absolute_path_policy`: the policy used for literal absolute container
  workdirs

If you want the same scheduling payload from the execution route without
starting the run, use preflight:

```bash
bijux-dag run ./pipelines/main.dag.json \
  --json \
  --out ./runs \
  --run-id rehearsal-main \
  --preflight-only \
  --explain-scheduling
```

Use the execution-cost estimate before a long run when you need to answer three
operator questions up front:

- how much of the graph is actually going to execute
- where the dependency bottleneck is
- whether resource demand, non-cacheable nodes, or aggressive timeout/retry
  settings make the run more expensive than it first looks

When a node carries `params.estimated_duration_ms`, the planner uses that value
to weight the reported `critical_path`. Nodes without an estimate fall back to
`1`, and the payload reports both the chosen path and how many nodes on that
path relied on the unit fallback.

## Rerun Everything Downstream Of A Node

When a parent run already exists and the operator knows which node should be
treated as the restart boundary, use `--from-node` instead of rebuilding the
entire graph mentally.

```bash
bijux-dag plan explain ./pipelines/main.dag.json \
  --json \
  --from-node train

bijux-dag replay ./runs/run-20260406-01 \
  --json \
  --out ./runs/replay-train \
  --from-node train
```

The downstream rerun contract is:

- the named node is included exactly once, by exact node id
- every descendant in the graph is included deterministically
- replay reexecutes the selected closure instead of satisfying it from stale
  replay cache reuse
- nodes outside the closure stay omitted and are reported as outside the
  requested downstream rerun boundary
- `--from-node` is exclusive with `--select`, `--exclude`, and
  `--dependency-closure`

## Compare Two Graph Versions Before Running

When an operator needs to understand whether a DAG edit is cosmetic or changes
execution, use `plan diff` against the two graph files instead of inferring the
impact from source control alone.

```bash
bijux-dag plan diff \
  ./pipelines/main-before.dag.json \
  ./pipelines/main-after.dag.json \
  --json
```

The diff contract reports:

- added and removed nodes
- node ids with changed params
- node ids with changed outputs
- node ids with changed resources
- node ids with changed retry or timeout policy
- added and removed dependencies
- whether drift is metadata-only or execution-affecting

Metadata-only drift means the graph fingerprint changed while the execution
fingerprint stayed stable. Execution-affecting drift means the planned runtime
surface changed and should be reviewed as a real workflow mutation.

When the operator needs a yes-or-no answer instead of a raw diff, use
`plan equivalence` against the same pair of graph files.

```bash
bijux-dag plan equivalence \
  ./pipelines/main-before.dag.json \
  ./pipelines/main-after.dag.json \
  --json
```

The equivalence contract reports:

- whether the graphs are execution-equivalent
- whether canonical graph identity stayed equal
- whether the execution fingerprint stayed equal
- which metadata drift was ignored to preserve equivalence
- the exact execution-affecting causes when equivalence fails

This keeps cosmetic metadata edits separate from real workflow mutations, and
it does not over-claim safety when planner-visible execution drift exists even
if the current execution fingerprint remains unchanged.

## Drive Scheduled Runs From Typed Trigger Context

When scheduled work needs graph inputs from trigger metadata, declare the input
contract in the schedule registry and submit explicit trigger payloads instead
of encoding that context indirectly in DAG defaults or ad hoc environment
state.

A submit input file can carry manual arguments, event payloads, dependency
completions, and signal payloads:

```json
{
  "now_unix_ms": 1762387200000,
  "manual_requests": [
    {
      "request_id": "manual-001",
      "schedule_id": "manual-ops",
      "requested_unix_ms": 1762387200000,
      "arguments": {
        "region": "eu-west-1"
      }
    }
  ],
  "events": [
    {
      "event_id": "evt-001",
      "event_type": "dataset.ready",
      "source": "catalog",
      "occurred_unix_ms": 1762387260000,
      "payload": {
        "tenant": "atlas",
        "batch": 7
      }
    }
  ],
  "signals": [
    {
      "signal_id": "sig-001",
      "signal_name": "refresh-cache",
      "occurred_unix_ms": 1762387320000,
      "payload": {
        "tenant": "atlas"
      }
    }
  ]
}
```

Submit the registry and trigger inputs together:

```bash
bijux-dag schedule submit ./ops/schedule-registry.json \
  ./ops/schedule-inputs.json \
  --out ./artifacts/schedule-ledger.json
```

The schedule-input binding contract is:

- trigger-derived values are normalized against the declared graph input types
- manual submissions can supply `manual_requests[].arguments` for bound inputs
- event and signal schedules can bind either the full payload or a JSON Pointer
  inside the payload
- dependency-triggered schedules can bind upstream run ids and completion
  status
- invalid or missing bindings suppress submission instead of creating a run
  request with partial graph input state

## Control A Historical Backfill

When an operator needs to replay a bounded historical window, treat the
backfill as a durable operation instead of a loose list of manual reruns.

Plan the operation from the schedule registry first:

```bash
bijux-dag schedule backfill plan ./ops/schedule-registry.json \
  --schedule-id historical-catalog \
  --planned-unix-ms 1762387200000 \
  --out ./artifacts/backfill-state.json
```

Advance the operation with explicit throttling inputs:

```bash
bijux-dag schedule backfill advance \
  ./artifacts/backfill-state.json \
  ./ops/backfill-advance-request.json \
  --out ./artifacts/backfill-state.json
```

Pause, resume, or cancel the same durable state file when operator control is
needed:

```bash
bijux-dag schedule backfill pause ./artifacts/backfill-state.json \
  --at-unix-ms 1762387500000 \
  --reason "hold while downstream catalog is degraded" \
  --out ./artifacts/backfill-state.json

bijux-dag schedule backfill resume ./artifacts/backfill-state.json \
  --at-unix-ms 1762388400000 \
  --out ./artifacts/backfill-state.json

bijux-dag schedule backfill cancel ./artifacts/backfill-state.json \
  --at-unix-ms 1762389000000 \
  --reason "superseded by corrected source snapshot" \
  --out ./artifacts/backfill-state.json
```

The backfill control contract is:

- time-window expansion is deterministic and advances in one-minute request
  slots across the declared window
- partition-list backfills expand deterministically within each time slot
- `max_parallelism` limits concurrent submitted or running backfill work
- the advance request applies live-load throttling before dispatching new runs
- failure policy controls whether a failed backfill run continues, pauses, or
  cancels remaining queued work
- pause and cancel stop new dispatches without pretending that already
  submitted runs were never issued

## Run Only The Prerequisites For A Target Node

When the operator wants the minimal execution closure required to reach one
target node, use `--to-node` instead of manually translating dependencies into
selectors.

```bash
bijux-dag plan explain ./pipelines/main.dag.json \
  --json \
  --to-node publish

bijux-dag run ./pipelines/main.dag.json \
  --json \
  --out ./runs \
  --run-id publish-prereqs \
  --to-node publish \
  --preflight-only
```

The upstream target contract is:

- the named node is included exactly once, by exact node id
- every required ancestor is included deterministically
- unrelated nodes stay omitted and are reported as outside the requested
  target boundary
- plan output distinguishes `selected_by_to_node` from
  `selected_by_upstream_closure`
- `--to-node` is exclusive with `--select`, `--exclude`, and
  `--dependency-closure`

## Code Anchors

- `crates/bijux-dag-app/src/routes/run_routes.rs`
- `crates/bijux-dag-app/src/routes/inspect_routes.rs`
- `crates/bijux-dag-app/src/routes/replay_routes.rs`
- `crates/bijux-dag-app/src/routes/diff_routes.rs`
- `crates/bijux-dag-app/src/routes/schedule_routes.rs`

## Reading Rule

Use this page when the question is not which DAG command exists, but which
sequence turns a run into something you can defend with evidence.

## Next Reads

- [Common Workflows](../operations/common-workflows.md)
- [Failure Recovery](../operations/failure-recovery.md)
- [Review Checklist](../quality/review-checklist.md)
