---
title: Operator Workflows
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Operator Workflows

A defensible DAG workflow moves from authored intent to retained evidence
without skipping validation or inferring success from console output.

The baseline is:

```text
validate -> plan -> run -> inspect -> replay or compare
```

Not every run needs replay or comparison. Every operational claim does need
evidence appropriate to that claim.

## Baseline Run

```bash
bijux-dag validate ./pipelines/main.dag.json
bijux-dag plan explain ./pipelines/main.dag.json --out ./runs
bijux-dag run ./pipelines/main.dag.json --out ./runs --run-id main-20260719
bijux-dag explain ./runs/main-20260719
bijux-dag runs inspect main-20260719 --root ./runs
```

Use this sequence to answer five different questions:

| Step | Question | Evidence |
| --- | --- | --- |
| validate | is the authored graph accepted? | validation result and diagnostics |
| plan | what will execute under the selected policy? | resolved selection, dependencies, resources, and path preview |
| run | did execution reach a terminal outcome? | finalized run directory and terminal summary |
| explain | why did the run or node reach that outcome? | retained events, attempts, cache decisions, and failure reasons |
| inspect | do retained files and indexes support the claim? | manifest, node traces, output indexes, and artifact verification |

The compact post-run summary is an orientation aid. The retained run directory
is the evidence boundary. Use [Run Evidence Layout](run-evidence-layout.md)
before making claims from individual files.

## Decide What To Do Next

| Operator question | Next action | Owning guide |
| --- | --- | --- |
| Which runs are active or recently completed? | `bijux-dag runs history --root ./runs` | [Run Evidence Layout](run-evidence-layout.md) |
| Why did this run fail? | `bijux-dag runs explain-failure <run-id> --root ./runs` | [Failure Recovery](../operations/failure-recovery.md) |
| Did two retained runs diverge? | `bijux-dag runs compare <before> <after> --root ./runs --json` | [Reproducibility Model](reproducibility-model.md) |
| Is the local cache trustworthy? | `bijux-dag --json cache verify --cache-dir ./.bijux/cache` | [Cache Behavior Workflow](../operations/cache-behavior-workflow.md) |
| What will a graph edit change? | `bijux-dag plan diff <before> <after> --json` | [Graph Schema](graph-schema.md) |
| Can prior evidence reproduce the result? | `bijux-dag replay <run-dir> --out <root>` | [Reproducibility Model](reproducibility-model.md) |
| Should an active run stop dispatching work? | `bijux-dag runs stop <run-id> --root ./runs` | [Failure Recovery](../operations/failure-recovery.md) |

Use JSON output for automation. Human output is optimized for diagnosis and
may change presentation without changing the structured contract.

## Preview Before Execution

Preview when paths, selection, or resource limits matter:

```bash
bijux-dag plan explain ./pipelines/main.dag.json \
  --json \
  --out ./runs \
  --run-id rehearsal-main \
  --jobs 4 \
  --cpu-budget 4 \
  --memory-budget-mb 8192 \
  --resource-capacity database_slot=1
```

The preview should make these decisions visible before work starts:

- selected nodes and dependency closure;
- staging and final run paths;
- resolved node-local path bindings;
- critical path and scheduling constraints;
- resource claims that cannot be satisfied;
- cache, timeout, and retry exposure.

`run --preflight-only --explain-scheduling` exposes the execution route's
equivalent preflight boundary. A preview proves interpretation, not successful
execution.

## Select An Honest Boundary

Use `--to-node <node>` when the goal is one target plus its required ancestors.
Use `--from-node <node>` during replay when the goal is one restart boundary
plus its descendants.

```bash
bijux-dag plan explain ./pipelines/main.dag.json --json --to-node publish
bijux-dag replay ./runs/main-20260719 \
  --json \
  --out ./runs/replay-train \
  --from-node train
```

Before downstream replay, the runtime verifies retained upstream artifacts
that cross into the selected closure. Selection is not permission to ignore a
corrupt dependency.

Both boundary flags are exclusive with selectors that would create an
ambiguous closure. Consult the
[Generated CLI Reference](generated-cli-reference.md) for exact flag
compatibility.

## Diagnose Without Guessing

### Failure And Retry

Start with the causal node and its attempt evidence:

- `nodes/<node-id>/attempts.json` records per-attempt decisions;
- `run.log.jsonl` and `observability.timeline.json` retain retry scheduling and
  exhaustion events;
- node explain output distinguishes causal failure from downstream fallout.

Do not treat blocked, skipped, cancelled, and failed nodes as interchangeable.
Their reason codes determine whether recovery should change the graph,
environment, policy, or selected replay boundary.

### Cache

Run stable cache verification before deleting entries. When one node needs a
deeper explanation, `why-cache-missed` is a tested explicit-path diagnostic,
but it remains outside the v0.4.0 stable help surface.

The diagnosis must distinguish changed identity, absent exact entries, and
integrity refusal. [Cache Behavior Workflow](../operations/cache-behavior-workflow.md)
shows all three against retained evidence.

### Comparison

Use retained-run comparison to locate the first provable divergence in status,
inputs, selection, fingerprints, or output hashes. Use detailed `diff` only
when the question requires artifact, policy, provenance, or node-level depth.

Equality of names or paths is not reproducibility proof. Read
[Reproducibility Model](reproducibility-model.md) for graph, plan, execution,
environment, cache, and output identity.

## Specialized Workflows

The overview does not duplicate procedures with stronger evidence:

| Situation | Procedure |
| --- | --- |
| real container engine, mounts, image identity, and retained streams | [Container Packaging Workflow](../operations/container-packaging-workflow.md) |
| branch decision, skipped lane, join trigger, and replay stability | [Branching Bulletin Workflow](../operations/branching-bulletin-workflow.md) |
| retry exhaustion, approval boundary, and repaired replay | [Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md) |
| warm cache, selective invalidation, and corruption refusal | [Cache Behavior Workflow](../operations/cache-behavior-workflow.md) |
| evidence-backed publication and artifact promotion | [Evidence-Backed Bulletin Workflow](../operations/evidence-backed-bulletin-workflow.md) |

Schedule, queue, dependency-trigger, event-lineage, and backfill routes are
internal in v0.4.x. They require `BIJUX_DAG_ENABLE_INTERNAL=1` and do not become
stable merely because maintainers can execute them:

- [Scheduled Catalog Refresh Workflow](../operations/scheduled-catalog-refresh-workflow.md)
- [Historical Catalog Backfill Workflow](../operations/historical-catalog-backfill-workflow.md)

## Completion Record

Before presenting a run as evidence, record:

- graph source and graph identity;
- effective inputs and selection;
- run ID and terminal status;
- output indexes and verification result;
- cache or replay decisions relevant to the claim;
- environment or backend limitations;
- comparison baseline when claiming equivalence or divergence.

If retained evidence is incomplete, state the gap. Do not reconstruct missing
proof from logs or operator memory.

## Code Anchors

- run routing: `crates/bijux-dag-app/src/routes/run_routes.rs`
- inspection: `crates/bijux-dag-app/src/routes/inspect_routes.rs`
- replay: `crates/bijux-dag-app/src/routes/replay_routes.rs`
- comparison: `crates/bijux-dag-app/src/routes/diff_routes.rs`

## Next Reads

- [Common Workflows](../operations/common-workflows.md)
- [Run Evidence Layout](run-evidence-layout.md)
- [Reproducibility Model](reproducibility-model.md)
- [Failure Recovery](../operations/failure-recovery.md)
- [Review Checklist](../quality/review-checklist.md)
