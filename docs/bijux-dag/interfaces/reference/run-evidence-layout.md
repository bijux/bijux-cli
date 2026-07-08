---
title: Run Evidence Layout
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Run Evidence Layout

This reference describes the retained filesystem layout for one finalized
`bijux-dag` run.

Use it when you need to know which files are authoritative, which files are
derived, and where node-level evidence is kept after execution finishes.

The examples on this page are based on the checked-in run-directory snapshots in
`crates/bijux-dag-app/tests/snapshots/` together with the live storage models in
`crates/bijux-dag-artifacts/src/storage/models.rs` and
`crates/bijux-dag-artifacts/src/lib.rs`.

## Directory shape

A retained run directory is named `run-<run_id>`.

The staging directory used during execution is `run.tmp-<run_id>`, but
operators normally work only with the finalized `run-<run_id>` tree.

```text
run-<run_id>/
├── manifest.json
├── graph.snapshot.json
├── outputs/
│   └── index.json
├── nodes/
│   └── <node_id>/
│       ├── trace.json
│       ├── attempts.json
│       ├── resolved_params.json
│       ├── stdout.log
│       ├── stderr.log
│       ├── inputs/
│       │   ├── index.json
│       │   └── <source_node_id>/<input_port>
│       ├── outputs/
│       │   ├── index.json
│       │   └── <declared output paths>
│       └── attempts/
│           └── <attempt>/
│               ├── stdout.log
│               └── stderr.log
├── lineage.snapshot.json
├── provenance.json
├── observability.events.json
├── observability.timeline.json
├── run.log.jsonl
└── run.schema.json
```

Standard runs can also retain supporting files such as:

- `manifest.finalized.json`
- `.run-complete.json` or `.run-incomplete.json`
- `run.snapshot.json`
- `run-log.index.json`
- `run.audit.json`
- `scheduler.checkpoint.json`
- `failure-propagation.json`
- `observability.metrics.json`
- `observability.root-causes.json`
- `observability.graph-visualization.json`
- `observability.lineage-visualization.json`

Those supporting files are useful for audit, repair, and inspection, but the
authoritative run evidence still starts with the manifest, graph snapshot,
node traces, indexes, event log, and timeline.

## Root evidence files

### `manifest.json`

`manifest.json` is the retained run summary.

It records:

- run identity such as `run_id`, `status`, and timestamps
- graph identity through `spec`, `graph_fingerprint`, and `graph_snapshot`
- planner and execution identity through `planner_contract_version`,
  `planner_fingerprint`, `execution_fingerprint`, and `evidence_fingerprint`
- adapter inventory and node outcome counts
- run-level output summaries from `outputs/index.json`
- policy, cache, timeout, and run metadata

This is the first file to open when the question is "what happened in this
run?"

### `graph.snapshot.json`

`graph.snapshot.json` is the persisted authored graph plus its resolved graph
fingerprint.

It is the reference surface for:

- declared nodes and edges
- graph-scoped inputs
- output contracts
- retry, timeout, resource, branch, and cache declarations

When operator tools need planned structure after a run completes, they read the
retained graph snapshot instead of reconstructing intent from logs.

### `outputs/index.json`

`outputs/index.json` is the run-level artifact index.

It aggregates the outputs retained by the run and records stable evidence per
artifact, including:

- `name`
- `path`
- `kind`
- `media_type`
- `size_bytes`
- `sha256`
- `node_id`
- `node_fingerprint`
- `promotable`

This file answers "which final artifacts did the run retain?" without forcing a
consumer to scan every node directory.

### `provenance.json` and `lineage.snapshot.json`

These files retain run-wide provenance and lineage context.

- `provenance.json` records the source and identity envelope for the run
- `lineage.snapshot.json` keeps the run-level lineage material used by replay,
  comparison, and artifact identity work

### `observability.events.json`, `observability.timeline.json`, and `run.log.jsonl`

These files retain the chronological execution record.

- `observability.events.json` is the structured event stream
- `observability.timeline.json` is the timeline-oriented summary for run and
  node transitions
- `run.log.jsonl` is the append-only event log used by audit and repair flows

Use the timeline when sequence and duration matter, and use the event log when
the exact retained event stream matters.

### `run.schema.json`

`run.schema.json` is the run-directory schema index.

It points to the live schema surfaces for:

- `manifest.json`
- `trace.json`
- `inputs/index.json`
- `outputs/index.json`
- lineage, timeline, and event-log versions

It also declares the required and optional root and node files for the current
retained run-directory format.

## Node evidence directories

Each executed node retains evidence under `nodes/<node_id>/`.

The node directory is the durable answer to "what happened for this one node?"

### `trace.json`

`trace.json` is the authoritative node outcome record.

It retains:

- node identity and final `status`
- start and finish timestamps
- terminal `attempt`
- node fingerprint and planner identity
- adapter identity and output schema version
- resolved resources, exit code, and declared outputs
- cache proof and cache identity when cache was involved
- branch, trigger-rule, skip, lifecycle, failure, and replay provenance data
- structured `stdout` and `stderr` evidence when available

### `attempts.json` and `attempts/<attempt>/`

`attempts.json` summarizes retry history for the node.

`attempts/<attempt>/` retains per-attempt logs:

- `stdout.log`
- `stderr.log`

Use the attempt subtree when the question is about retries, transient failures,
or per-attempt output rather than only the final terminal state.

### `resolved_params.json`

`resolved_params.json` keeps the execution-time parameter payload after graph
input binding, path binding, and node-output reference resolution.

This is the file to read when the question is "what did the runtime actually
hand to the adapter?"

### `stdout.log` and `stderr.log`

The node root also retains the latest node-level `stdout.log` and `stderr.log`.

Newer runs prefer the structured stdout and stderr evidence embedded in
`trace.json`, but these retained log files remain important for direct
inspection, cache capture, and repair flows.

## Input directories

Each node keeps its materialized inputs under `nodes/<node_id>/inputs/`.

The authoritative index is:

- `nodes/<node_id>/inputs/index.json`

Materialized payloads live under paths such as:

- `nodes/<node_id>/inputs/<source_node_id>/<input_port>`

The input index records:

- `local_path`
- `source_node_id`
- `source_node_fingerprint`
- `source_output_name`
- `source_sha256`
- `materialization_mode`

That split matters:

- the payload files are what the node consumed
- the index explains where each payload came from and why it is trusted

## Output directories

Each node keeps its retained outputs under `nodes/<node_id>/outputs/`.

The authoritative index is:

- `nodes/<node_id>/outputs/index.json`

Declared output payloads are retained at their contract paths beneath the node
output root, for example:

- `nodes/const1/outputs/out_const`
- `nodes/echo/outputs/out_echo`

The node output index uses the same evidence shape as the run-level output
index, but it stays scoped to one producer node.

## Execution work directories

During execution the runtime also allocates:

- `nodes/<node_id>/work/`
- `nodes/<node_id>/work/temp/`

Those directories are staging-time execution surfaces, not part of the durable
retained evidence contract. Finalized run snapshots are expected to preserve the
inputs, outputs, trace, and logs rather than the transient work tree.

## Code anchors

- `crates/bijux-dag-artifacts/src/lib.rs`
- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-app/tests/snapshots/run_dir_hello.json`
- `crates/bijux-dag-app/tests/snapshots/run_dir_cached_branch.json`

## Related references

- [Artifact Contracts](../artifact-contracts.md)
- [Node Inspection](node-inspection.md)
- [State and Persistence](../../architecture/state-and-persistence.md)
