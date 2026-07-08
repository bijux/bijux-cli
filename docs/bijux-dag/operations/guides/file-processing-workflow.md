---
title: File Processing Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# File Processing Workflow

This guide demonstrates one practical local workflow that validates source
files, normalizes their content, aggregates run metrics, and renders a final
promotable report artifact.

The workflow is backed by the repository example
`evidence/dag/authoring/examples/file-processing-report.dag.json` and the
sample input directory `evidence/dag/authoring/examples/file-processing-source`.

If you want the shortest repository command that proves this workflow before
you inspect each operator step, run:

```bash
make dag-demo
```

That command executes the same retained workflow this guide explains in detail,
using `artifacts/dag-demo/` as its run and cache root.

## What This Workflow Proves

- graph inputs drive a real run instead of only a toy constant graph
- warm cache reuse is visible on a second run
- artifact lineage is inspectable from the retained run
- a focused replay can rerun the final reporting boundary
- the final report artifact can be promoted into a deliverables root

## Prepare Inputs

Run these commands from repository root so the sample input directory can be
resolved explicitly:

```bash
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"
GRAPH_PATH="evidence/dag/authoring/examples/file-processing-report.dag.json"
RUN_ROOT="./artifacts/file-processing-runs"
CACHE_ROOT="./artifacts/file-processing-cache"
DELIVERABLES_ROOT="./artifacts/file-processing-deliverables"
```

The workflow requires `source_dir` as a runtime graph input. It is a required
path input because node processes execute from the run directory, not from the
repository root.

## Validate The Graph

```bash
bijux-dag validate "${GRAPH_PATH}"
```

## Run The Workflow

The first run writes four node stages:

1. `validate_files`
2. `transform_files`
3. `aggregate_metrics`
4. `render_report`

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id file-processing-source \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=Repository File Processing Report"
```

The final report lands at:

```text
artifacts/file-processing-runs/run-file-processing-source/nodes/render_report/outputs/report/report.md
```

## Inspect Run And Artifact Evidence

Inspect the run summary and the artifact registry:

```bash
bijux-dag explain "${RUN_ROOT}/run-file-processing-source"
bijux-dag artifact registry "${RUN_ROOT}/run-file-processing-source" --json
```

Inspect the final report artifact directly:

```bash
bijux-dag artifact-inspect \
  "${RUN_ROOT}/run-file-processing-source" \
  render_report:report.md
```

Inspect lineage for the retained run:

```bash
bijux-dag artifact lineage \
  "${RUN_ROOT}/run-file-processing-source" \
  --json
```

The lineage view should show the report artifact downstream of the summary
artifact, which itself depends on the normalized file directory.

## Show Warm Cache Reuse

Run the same workflow again with the same cache directory:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id file-processing-second \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=Repository File Processing Report"
```

On the warm run, `validate_files`, `transform_files`, and `aggregate_metrics`
should be reused from cache. `render_report` is intentionally marked
non-cacheable so the final published artifact is always regenerated from the
retained summary input.

## Rerun The Final Reporting Boundary

Use replay to rerun only the final reporting node while preserving ancestry
back to the original run:

```bash
bijux-dag replay --json \
  --source-run-id file-processing-source \
  --source-run-root "${RUN_ROOT}" \
  --out "${RUN_ROOT}" \
  --run-id file-processing-rerun \
  --from-node render_report
```

This focused replay keeps the original run as the parent run and reuses the
persisted upstream evidence for the selected rerun boundary.

## Promote The Final Report

Promote the final report into a deliverables root:

```bash
bijux-dag artifact promote \
  "${RUN_ROOT}/run-file-processing-source" \
  render_report:report.md \
  --deliverables-root "${DELIVERABLES_ROOT}" \
  --to release \
  --json
```

The promoted payload is copied into:

```text
artifacts/file-processing-deliverables/release/file-processing-source/render_report/report/payload/report.md
```

The source run manifest records the promotion summary so later inspection can
tell which run artifact became a deliverable.

## Reading Rule

Use this guide when you want one practical DAG workflow that exercises runtime
inputs, retained evidence, replay, cache reuse, and artifact promotion together
on the stable local operator surface.

## Next Reads

- [First-Run Tutorial](first-run-tutorial.md)
- [First Hour With Bijux Dag](first-hour-with-bijux-dag.md)
- [Common Workflows](../common-workflows.md)
- [Operator Workflows](../../interfaces/operator-workflows.md)
