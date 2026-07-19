---
title: Run Evidence Layout
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Run Evidence Layout

This reference identifies retained files for one finalized `bijux-dag` run and
states what each file can prove.

The live storage models and checked-in snapshots are the implementation
authority:

- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-artifacts/src/lib.rs`
- `crates/bijux-dag-app/tests/snapshots/run_dir_hello.json`
- `crates/bijux-dag-app/tests/snapshots/run_dir_cached_branch.json`

## Trust Order

| Evidence | Use |
| --- | --- |
| `manifest.json`, `graph.snapshot.json`, `run.schema.json` | establish run, graph, planner, and schema identity |
| `nodes/<node_id>/trace.json`, attempt records, input/output indexes | establish node outcome and materialized contracts |
| `outputs/index.json`, lineage, and provenance | establish artifact identity and dependency history |
| `run.log.jsonl`, events, and timeline | reconstruct ordered runtime observations |
| summaries and visualizations | orient investigation; do not replace underlying records |
| work and temporary directories | execution staging, never finalized proof |

A successful command envelope proves neither retained integrity nor replay
eligibility. Verify hashes and schemas before making consequential claims.

## Finalized Directory

Execution stages in `run.tmp-<run_id>` and finalizes to `run-<run_id>`.

```text
run-<run_id>/
├── manifest.json
├── graph.snapshot.json
├── run.schema.json
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
└── run.log.jsonl
```

Optional supporting files include `manifest.finalized.json`,
`.run-complete.json`, `.run-incomplete.json`, `run.snapshot.json`,
`run-log.index.json`, `run.audit.json`, `scheduler.checkpoint.json`,
`failure-propagation.json`, observability summaries, and `plan.json`.

## Root Evidence

### Run Identity

`manifest.json` is the first inspection point. It records:

- run ID, status, timestamps, and node outcome counts;
- graph, planner, execution, and evidence fingerprints;
- adapter inventory and selected policy;
- effective run inputs and cache configuration;
- output and promotion summaries.

The manifest records `cache_mode` and `cache_dir`; cache payloads live under
the configured cache root, not inside the run.

`graph.snapshot.json` preserves authored graph intent and graph identity.
Operator tools use it instead of reconstructing nodes, edges, inputs, output
contracts, branches, retries, or resources from logs.

`run.schema.json` indexes required and optional root/node files and their live
schema versions.

### Artifact Identity

`outputs/index.json` is the run-level artifact index. Entries identify output
name, producer node, relative path, kind, media type, size, digest, producer
fingerprint, and promotion eligibility.

`provenance.json` retains the run source and identity envelope.
`lineage.snapshot.json` retains run-wide dependency and artifact lineage used by
replay and comparison.

### Ordered Observation

- `observability.events.json` retains structured events;
- `observability.timeline.json` retains normalized lifecycle order;
- `run.log.jsonl` retains the append-only audit and repair stream.

Failed, skipped, cached, cancelled, timed-out, and successful outcomes remain
distinct. Sequence evidence explains what the runtime observed; it does not
override terminal state or artifact verification.

### Scheduler State

When present, `scheduler.checkpoint.json` records one scheduler-loop boundary:
ready queue, scheduled batch, resource blocks, inflight ownership, completed
states, and decision reason.

### Optional Plan

`plan.json` may preserve the lowered plan under
`configs/dag/schema/execution_plan.schema.json`. Standard local runs do not
promise it. Planner identity is always carried in the manifest, while graph
intent is retained in `graph.snapshot.json`.

## Node Evidence

Every executed or terminally classified node has a directory under
`nodes/<node_id>/`.

| Path | Meaning |
| --- | --- |
| `nodes/<node_id>/trace.json` | authoritative node status, timing, identity, policy, outputs, cache, branch, failure, and replay record |
| `nodes/<node_id>/attempts.json` | retry and attempt summary |
| `nodes/<node_id>/attempts/<attempt>/` | per-attempt stdout and stderr |
| `nodes/<node_id>/resolved_params.json` | values handed to the adapter after reference and path resolution |
| `nodes/<node_id>/stdout.log` | terminal/latest retained stdout |
| `nodes/<node_id>/stderr.log` | terminal/latest retained stderr |

Structured stream evidence in `trace.json` is preferable for automation.
Retained log files remain useful for direct diagnosis and repair.

## Materialized Inputs

`nodes/<node_id>/inputs/index.json` maps each consumed payload to:

- local path;
- source node and source output;
- source node fingerprint and content digest;
- materialization mode (`copy`, `hardlink`, or `symlink`).

Payloads live below
`nodes/<node_id>/inputs/<source_node_id>/<input_port>`. The payload is what the
adapter consumed; the index is why that payload can be attributed upstream.

## Retained Outputs

`nodes/<node_id>/outputs/index.json` maps declared output contracts to retained
payloads and digests. Payloads stay at their declared relative paths beneath
the node output root.

The node index proves one producer boundary. The root `outputs/index.json`
aggregates run-level artifact visibility.

## Cache Evidence

Cache entries are reusable node results, not miniature run directories:

```text
<cache_root>/<cache_key>/
├── manifest.json
├── meta.json
├── outputs/
│   ├── index.json
│   └── <payloads>
└── logs/
    ├── stdout.log
    ├── stderr.log
    └── trace.json
```

The cache manifest binds `manifest_version`, cache key, node ID, and output
contracts. `meta.json` records explainability and integrity inputs such as node,
environment, lineage, params, command, adapter, policy, execution-contract,
and backend fingerprints.

Reuse is valid only when the exact entry remains identity-compatible and its
retained files verify.

## Promotion Evidence

Promotions are retained inside the source run:

```text
run-<run_id>/promotions/
├── index.json
└── <promotion-record>.json
```

`promotions/index.json` is the run-local ledger. A record binds source run,
node, output, digest, destination, source/target environments, timestamp, and
lineage. The manifest carries a compact summary at
`run_summary.promoted_outputs`.

Use the summary for discovery and the promotion ledger for audit.

## Transient Work

`nodes/<node_id>/work/` and `nodes/<node_id>/work/temp/` are execution
scratch. They are not durable evidence and must not be required for replay,
verification, or post-run explanation.

## Choose Evidence By Question

| Question | Start with | Corroborate with |
| --- | --- | --- |
| What ran and how did it finish? | `manifest.json` | graph snapshot and timeline |
| What happened to one node? | `trace.json` | attempts, streams, and resolved params |
| What input did a node consume? | input index | materialized payload and upstream output index |
| Which artifacts survived? | root output index | node output index and content digest |
| Why was cache reused or refused? | node cache proof | cache manifest and `meta.json` |
| What was promoted? | `run_summary.promoted_outputs` | `promotions/index.json` and record |
| Can this run be replayed? | manifest and retained inputs | [Reproducibility Model](reproducibility-model.md) |

## Related References

- [Artifact Contracts](artifact-contracts.md)
- [Node Inspection](node-inspection.md)
- [State And Persistence](../architecture/state-and-persistence.md)
- [Reproducibility Model](reproducibility-model.md)
