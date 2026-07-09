---
title: Runnable Examples
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Runnable Examples

This page is the public examples index for `bijux-dag` v0.4.0.

Use it when the question is "which repository example should I run for this
behavior, and what should I expect to see when it works?"

Every example listed here is backed by a checked-in graph, a repository-owned
fixture set, or a repository-tested guide. The index stays on the honest
release boundary: local-first execution, retained evidence, cache and replay
proof, branch visibility, and container packaging when the required engine is
present.

## Example Map

| Example | Primary surface | Graph or guide | Expected outputs |
| --- | --- | --- | --- |
| minimal hello DAG | validate and run a tiny graph | `evidence/dag/authoring/examples/hello.dag.json` | retained run directory, constant output, shell output |
| file-processing DAG | run one practical local report workflow | `evidence/dag/authoring/examples/file-processing-report.dag.json` | rendered report, artifact registry entry, replayable run per [Replay Contract](../../../spec/REPLAY_CONTRACT.md) |
| cache demo | prove warm reuse and selective invalidation | [Cache Behavior Workflow](../../operations/guides/cache-behavior-workflow.md) | warm cache hits, changed-input invalidation, cache-miss explanation |
| failure demo | prove retry evidence and focused repair | [Compliance-Gated Bulletin Workflow](../../operations/guides/compliance-gated-bulletin-workflow.md) | retry attempt record, failed approval boundary, repaired verified run |
| replay demo | rerun a selected boundary from retained evidence | [First-Run Tutorial](../../operations/guides/first-run-tutorial.md) | replay proof, node-scoped rerun diff, strict verification success |
| branch demo | prove selected and skipped lanes stay visible | `evidence/dag/authoring/examples/audience-branch-bulletin.dag.json` | branch decision artifact, one rendered lane, one skipped lane |
| container demo | prove mounted inputs, retained outputs, and recorded engine identity | `evidence/dag/authoring/examples/release-note-bundle.dag.json` | bundled release note, container summary, recorded image digest |

## Minimal Hello DAG

Graph:
`evidence/dag/authoring/examples/hello.dag.json`

Run:

```bash
bijux-dag validate evidence/dag/authoring/examples/hello.dag.json
bijux-dag run --json evidence/dag/authoring/examples/hello.dag.json \
  --out ./artifacts/hello-runs \
  --run-id hello-example
```

Expected outputs:

- the validation command succeeds without additional graph inputs
- the run envelope returns `ok: true` and reports
  `./artifacts/hello-runs/run-hello-example`
- the retained run contains node output directories for `const1` and `echo`
- the shell node writes the retained `out_echo` artifact with the text
  `from shell`

## File-Processing DAG

Graph:
`evidence/dag/authoring/examples/file-processing-report.dag.json`

Run:

```bash
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"
bijux-dag run --json evidence/dag/authoring/examples/file-processing-report.dag.json \
  --out ./artifacts/file-processing-runs \
  --run-id file-processing-source \
  --cache readwrite \
  --cache-dir ./artifacts/file-processing-cache \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=Examples Index Report"
```

Expected outputs:

- the retained manifest records `source_dir` and `report_title` under
  `run_metadata.graph_inputs`
- the final report is materialized at
  `nodes/render_report/outputs/report/report.md`
- the report includes the supplied title, processed file count, and aggregate
  line totals
- `bijux-dag artifact registry ./artifacts/file-processing-runs/run-file-processing-source --json`
  lists the final report as a retained artifact

Guide:
[File Processing Workflow](../../operations/guides/file-processing-workflow.md)

## Cache Demo

Graph family:
`evidence/dag/authoring/examples/regional-sales-pipeline.dag.json`

Primary proof path:

```bash
bijux-dag run --json evidence/dag/authoring/examples/regional-sales-pipeline.dag.json \
  --out ./artifacts/regional-sales-runs \
  --run-id regional-sales-warm \
  --cache readwrite \
  --cache-dir ./artifacts/regional-sales-cache \
  --input "orders_csv=${ORDERS_CSV}" \
  --input "targets_json=${TARGETS_JSON}" \
  --input "report_title=Regional Revenue Attainment"

bijux-dag --json why-cache-missed \
  --run-dir ./artifacts/regional-sales-runs/run-regional-sales-updated \
  --node clean_orders \
  --cache-dir ./artifacts/regional-sales-cache
```

Expected outputs:

- the warm run reports cached reuse for the independent retained stages
- a changed orders input invalidates only the dependent branch
- the targets branch stays cached across the changed run
- `why-cache-missed` surfaces the changed input hash or cache-identity reason
  instead of a generic miss

Guide:
[Cache Behavior Workflow](../../operations/guides/cache-behavior-workflow.md)

## Failure Demo

Graph family:
`evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json`

Primary proof path:

```bash
bijux-dag run --json evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json \
  --out ./artifacts/compliance-gated-runs \
  --run-id compliance-gated-source \
  --input "source_note=${SOURCE_NOTE}" \
  --input "retry_plan=$(pwd)/artifacts/compliance-gated-retry-plan.json" \
  --input "publication_gate=$(pwd)/artifacts/compliance-gated-publication-gate.json"

bijux-dag replay --json --source-run-id compliance-gated-source \
  --source-run-root ./artifacts/compliance-gated-runs \
  --out ./artifacts/compliance-gated-runs \
  --run-id compliance-gated-repaired \
  --from-node validate_publication_gate
```

Expected outputs:

- the first run records transient retry behavior on `fetch_compliance_gate`
- approval failure stays visible at `validate_publication_gate`
- the repaired replay reruns only the failed approval boundary and downstream
  publication step
- `bijux-dag verify --json ./artifacts/compliance-gated-runs/run-compliance-gated-repaired --strict`
  succeeds after repair

Guide:
[Compliance-Gated Bulletin Workflow](../../operations/guides/compliance-gated-bulletin-workflow.md)

## Replay Demo

Primary proof path:

```bash
bijux-dag replay --json --source-run-id first-run-tutorial-cold \
  --source-run-root ./artifacts/first-run-tutorial-runs \
  --out ./artifacts/first-run-tutorial-runs \
  --run-id first-run-tutorial-replay \
  --from-node render_report

bijux-dag verify --json \
  ./artifacts/first-run-tutorial-runs/run-first-run-tutorial-replay \
  --strict
```

Expected outputs:

- the replay envelope contains `replay_proof`
- the replay response reports `node_rerun_diff.node_id` as `render_report`
- the replayed run directory records `parent_run_id` and `source_run_id`
- strict verification succeeds on the replayed run

Guide:
[First-Run Tutorial](../../operations/guides/first-run-tutorial.md)

## Branch Demo

Graph:
`evidence/dag/authoring/examples/audience-branch-bulletin.dag.json`

Run:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/audience-branch-source/team-update.md"
bijux-dag run --json evidence/dag/authoring/examples/audience-branch-bulletin.dag.json \
  --out ./artifacts/audience-branch-runs \
  --run-id audience-branch-technical \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=technical"
```

Expected outputs:

- `choose_audience_lane` writes a retained branch decision artifact
- `render_technical_bulletin` succeeds while
  `render_executive_bulletin` is retained as skipped
- `publish_bulletin` emits the selected bulletin plus
  `publish/selection.json` with `selected_lane: technical`
- replaying the run from the retained source keeps the same lane selection

Guide:
[Branching Bulletin Workflow](../../operations/guides/branching-bulletin-workflow.md)

## Container Demo

Graph:
`evidence/dag/authoring/examples/release-note-bundle.dag.json`

Run:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/release-note-source/weekly-update.md"
bijux-dag run --json evidence/dag/authoring/examples/release-note-bundle.dag.json \
  --out ./artifacts/release-note-bundle-runs \
  --run-id release-note-bundle \
  --input "source_note=${SOURCE_NOTE}" \
  --input "bundle_label=Release Brief"
```

Expected outputs:

- `prepare_note` copies the retained source note into node inputs
- `package_bundle` writes `bundle/release-note.txt` and
  `bundle/container-summary.json`
- the retained trace records container engine details and the configured image
  digest
- if the container engine is unavailable, the run fails as a clear
  infrastructure error instead of a silent skip

Guide:
[Container Packaging Workflow](../../operations/guides/container-packaging-workflow.md)

## Next Reads

- [Entrypoints and Examples](../entrypoints-and-examples.md)
- [Executable Recipes](../executable-recipes.md)
- [Operator Workflows](../operator-workflows.md)
- [First-Run Tutorial](../../operations/guides/first-run-tutorial.md)
- [Cache Behavior Workflow](../../operations/guides/cache-behavior-workflow.md)
- [Compliance-Gated Bulletin Workflow](../../operations/guides/compliance-gated-bulletin-workflow.md)
