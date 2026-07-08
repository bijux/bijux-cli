---
title: Evidence-Backed Bulletin Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Evidence-Backed Bulletin Workflow

This guide demonstrates one retained workflow family that carries the core
`bijux-dag` operator story in a single domain path:

- runtime graph inputs choose both the source note and the branch lane
- typed outputs produce retained branch evidence and a promotable bulletin
- warm cache reuse is visible on the same workflow family
- changed-input comparison identifies the branch and content drift
- focused replay proves the publication boundary can be rerun without branch
  decision drift
- strict verification and artifact hashing confirm the retained evidence
- artifact promotion copies the final bulletin into a deliverables root

The workflow is backed by
`evidence/dag/authoring/examples/audience-branch-bulletin.dag.json` plus the
source fixtures under `evidence/dag/authoring/examples/audience-branch-source`.

## Prepare Inputs

Run these commands from repository root:

```bash
GRAPH_PATH="evidence/dag/authoring/examples/audience-branch-bulletin.dag.json"
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/audience-branch-source/team-update.md"
REVISED_NOTE="$(pwd)/evidence/dag/authoring/examples/audience-branch-source/team-update-revised.md"
RUN_ROOT="./artifacts/evidence-backed-bulletin-runs"
CACHE_ROOT="./artifacts/evidence-backed-bulletin-cache"
DELIVERABLES_ROOT="./artifacts/evidence-backed-bulletin-deliverables"
```

## Validate The Graph

```bash
bijux-dag validate "${GRAPH_PATH}"
```

## Run The First Technical Bulletin

This first run writes retained evidence for one selected branch lane plus the
final publication output:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id branch-bulletin-cold \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=technical"
```

Inspect the retained publication payload:

```text
artifacts/evidence-backed-bulletin-runs/run-branch-bulletin-cold/nodes/publish_bulletin/outputs/publish/bulletin.md
```

You should see a technical bulletin with the selected lane recorded in
`publish/selection.json`.

## Show Warm Cache Reuse

Run the same workflow again with the same cache directory and inputs:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id branch-bulletin-warm \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=technical"
```

On the warm run, `prepare_note`, `choose_audience_lane`, and
`render_technical_bulletin` should be reused from cache. The final
`publish_bulletin` node is intentionally regenerated so the published artifact
is rebuilt from retained upstream evidence.

## Inspect Artifact Hash And Lineage

Inspect the published bulletin artifact and verify its hash directly:

```bash
bijux-dag artifact registry "${RUN_ROOT}/run-branch-bulletin-cold" --json

bijux-dag artifact-inspect \
  "${RUN_ROOT}/run-branch-bulletin-cold" \
  publish_bulletin:bulletin.md \
  --json

bijux-dag hash artifact --json \
  "${RUN_ROOT}/run-branch-bulletin-cold/nodes/publish_bulletin/outputs/publish/bulletin.md"
```

Inspect the retained lineage snapshot:

```bash
bijux-dag artifact lineage \
  "${RUN_ROOT}/run-branch-bulletin-cold" \
  --json
```

The final bulletin should show upstream lineage through the selected render
node and the prepared source note.

## Compare A Changed Executive Run

Now change both the input note and the branch selector:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id branch-bulletin-updated \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "source_note=${REVISED_NOTE}" \
  --input "audience_mode=executive"
```

Compare the warm technical run against the updated executive run:

```bash
bijux-dag runs compare \
  branch-bulletin-warm \
  branch-bulletin-updated \
  --root "${RUN_ROOT}" \
  --json
```

The comparison should show:

- `input_values.changed_inputs` includes `source_note` and `audience_mode`
- `node_statuses.changed_nodes` includes both render lanes
- `output_hashes.changed_outputs` includes
  `publish_bulletin:publish/bulletin.md`
- `output_hashes.changed_outputs` includes
  `publish_bulletin:publish/selection.json`

This is the practical proof that the workflow records both content drift and
branch-selection drift in retained evidence.

## Prove Focused Replay At The Publication Boundary

Run one source bulletin through the executive lane, then replay only the
publication boundary with dependency closure and replay proof enabled:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id branch-bulletin-proof-source \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=executive"

bijux-dag replay --json \
  --source-run-id branch-bulletin-proof-source \
  --source-run-root "${RUN_ROOT}" \
  --out "${RUN_ROOT}" \
  --run-id branch-bulletin-replay \
  --select id:publish_bulletin \
  --dependency-closure \
  --prove
```

The replay proof should show:

- `replay_proof.equivalent = true`
- `replay_proof.fidelity_level = "strict_equivalent"`
- `replay_proof.branch_decision_drift_nodes = []`

This is the bounded rerun story: replay reexecutes only the selected closure
and proves that the retained branch decision stayed stable.

## Verify The Replay Strictly

```bash
bijux-dag verify --json \
  "${RUN_ROOT}/run-branch-bulletin-replay" \
  --strict
```

Strict verification should complete without root-level errors. Skipped branch
lanes remain visible as skipped evidence rather than being treated as silent
loss.

## Promote The Final Bulletin

Promote the updated executive bulletin into a deliverables root:

```bash
bijux-dag artifact promote \
  "${RUN_ROOT}/run-branch-bulletin-updated" \
  publish_bulletin:bulletin.md \
  --deliverables-root "${DELIVERABLES_ROOT}" \
  --to release \
  --json
```

The promoted payload lands at:

```text
artifacts/evidence-backed-bulletin-deliverables/release/branch-bulletin-updated/publish_bulletin/bulletin/payload/bulletin.md
```

## Failure Signals

When the input file path is wrong or empty, `prepare_note` fails immediately
with a direct file error instead of letting the workflow drift into a vague
publication failure. When the branch lane is not selected, the nonmatching
render node is retained with `skip_reason.reason =
"branch_decision_not_selected"` so the skipped lane remains inspectable.

## Reading Rule

Use this guide when you want one retained workflow family that demonstrates
cache reuse, changed-run attribution, replay proof, artifact integrity, and
promotion together on the stable `bijux-dag` operator surface.

## Next Reads

- [Branching Bulletin Workflow](branching-bulletin-workflow.md)
- [File Processing Workflow](file-processing-workflow.md)
- [Data Pipeline Workflow](data-pipeline-workflow.md)
- [Operator Workflows](../../interfaces/operator-workflows.md)
