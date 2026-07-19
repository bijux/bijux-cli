---
title: Branching Bulletin Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# Branching Bulletin Workflow

This guide demonstrates a real branch-backed DAG workflow that routes one
source note into an executive or technical publication lane, skips the
unselected branch, and joins the selected output into one retained bulletin.

The workflow is backed by
`evidence/dag/authoring/examples/audience-branch-bulletin.dag.json` and the
sample note under
`evidence/dag/authoring/examples/audience-branch-source/team-update.md`.

## What This Workflow Proves

- a typed graph input drives a real branch decision
- only the matching render lane executes
- the nonmatching branch is retained as an explicit skip, not silently ignored
- the join node succeeds with `none_failed` when one parent is `success` and
  the other is `skipped`
- replay keeps the same selected lane and reports no branch-decision drift

## Prepare The Run

Run these commands from repository root:

```bash
GRAPH_PATH="evidence/dag/authoring/examples/audience-branch-bulletin.dag.json"
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/audience-branch-source/team-update.md"
RUN_ROOT="./artifacts/audience-branch-runs"
```

The workflow accepts two graph inputs:

- `source_note`, a required path to the input note
- `audience_mode`, an enum with `executive` and `technical`

## Validate The Graph

```bash
bijux-dag validate "${GRAPH_PATH}"
```

## Run The Technical Lane

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id audience-branch-technical \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=technical"
```

The retained bulletin lands at:

```text
artifacts/audience-branch-runs/run-audience-branch-technical/nodes/publish_bulletin/outputs/publish/bulletin.md
```

## Inspect The Branch Decision

Inspect the branch node and the retained join trace:

```bash
bijux-dag explain "${RUN_ROOT}/run-audience-branch-technical"
cat "${RUN_ROOT}/run-audience-branch-technical/nodes/choose_audience_lane/trace.json"
cat "${RUN_ROOT}/run-audience-branch-technical/nodes/publish_bulletin/trace.json"
```

The retained evidence should show:

- `choose_audience_lane.branch_decision = "technical"`
- `render_technical_bulletin.status = "success"`
- `render_executive_bulletin.status = "skipped"`
- `render_executive_bulletin.skip_reason.reason = "branch_decision_not_selected"`
- `publish_bulletin.trigger_evaluation.trigger_rule = "none_failed"`

Inspect the selected output directly:

```bash
cat "${RUN_ROOT}/run-audience-branch-technical/nodes/publish_bulletin/outputs/publish/selection.json"
cat "${RUN_ROOT}/run-audience-branch-technical/nodes/publish_bulletin/outputs/publish/bulletin.md"
```

## Replay The Publication Boundary

Run the source workflow again through the executive lane, then replay the
publication boundary with dependency closure and replay proof enabled:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id audience-branch-source \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=executive"

bijux-dag replay --json \
  --source-run-id audience-branch-source \
  --source-run-root "${RUN_ROOT}" \
  --out "${RUN_ROOT}" \
  --run-id audience-branch-replay \
  --select id:publish_bulletin \
  --dependency-closure \
  --prove
```

This replay command reruns the publication boundary with the required upstream
closure and emits a replay proof in the JSON response.

## Inspect Replay Stability

Inspect the replay result:

```bash
cat "${RUN_ROOT}/run-audience-branch-replay/nodes/choose_audience_lane/trace.json"
cat "${RUN_ROOT}/run-audience-branch-replay/nodes/render_technical_bulletin/trace.json"
cat "${RUN_ROOT}/run-audience-branch-replay/nodes/publish_bulletin/outputs/publish/selection.json"
```

The replay proof and retained traces should show:

- `replay_proof.equivalent = true`
- `replay_proof.branch_decision_drift_nodes = []`
- `choose_audience_lane.branch_decision = "executive"`
- `render_technical_bulletin.status = "skipped"` during the executive replay
- `selection.json` still records the executive lane

## Reading Rule

Use this guide when the question is whether `bijux-dag` branch semantics are
actually useful at the operator surface instead of only represented in the
schema and runtime internals.

## Next Reads

- [Common Workflows](common-workflows.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Failure Recovery](failure-recovery.md)
- [Data Pipeline Workflow](data-pipeline-workflow.md)
