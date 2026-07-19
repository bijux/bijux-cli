---
title: Executable Examples
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Executable Examples

This page maps operator questions to checked-in graphs and retained evidence.
It has two trust levels:

- the example catalog names fixtures, commands, and expected observations;
- the marked CI recipes are extracted and executed by
  `docs_executable_recipes_contract.rs`.

An expected observation is not a pass claim until the named command has run
against the current binary. Stable commands stay on the visible operator
surface. The experimental explicit-path routes are labeled and remain outside
the stable compatibility promise.

[Release Boundary](../foundation/release-boundary.md) explains the command
lanes; `contracts/foundation/dag_release_truth_table.v1.json` is the
machine-readable classification authority.

## Choose An Example

| Question | Example | Strongest retained evidence |
| --- | --- | --- |
| Can the runtime validate and execute a tiny graph? | minimal hello DAG | finalized run and two node outputs |
| Can inputs become a verifiable report? | file-processing DAG | manifest inputs, report artifact, and output digest |
| Why did cache reuse or refusal occur? | cache demo | cache decision and `why-cache-missed` reason |
| Can a failed boundary be repaired without hiding attempts? | failure demo | attempt history, failed gate, and verified replay |
| Can one downstream boundary rerun from retained evidence? | replay demo | `replay_proof`, lineage, focused diff, and strict verification |
| Are selected and skipped branch lanes both visible? | branch demo | branch decision, skipped trace, and selected output |
| Does a real container step retain engine and image identity? | container demo | container trace and `container-summary.json` |

## Minimal Hello DAG

Use `evidence/dag/authoring/examples/hello.dag.json` to prove the smallest
graph and shell-output path:

```bash
bijux-dag validate evidence/dag/authoring/examples/hello.dag.json
bijux-dag run --json evidence/dag/authoring/examples/hello.dag.json \
  --out ./artifacts/hello-runs \
  --run-id hello-example
```

Expected outputs:

- a finalized `run-hello-example` directory;
- retained outputs for `const1` and `echo`;
- `out_echo` containing `from shell`.

## File-Processing DAG

Use `evidence/dag/authoring/examples/file-processing-report.dag.json` for a
typed-input report workflow:

```bash
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"
bijux-dag run --json evidence/dag/authoring/examples/file-processing-report.dag.json \
  --out ./artifacts/file-processing-runs \
  --run-id file-processing-source \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=Examples Index Report"
```

Expected outputs:

- effective inputs retained in the run manifest;
- `nodes/render_report/outputs/report/report.md`;
- a report entry in the artifact registry.

The full procedure is [File Processing Workflow](../operations/file-processing-workflow.md).

## Cache Demo

Use `evidence/dag/authoring/examples/regional-sales-pipeline.dag.json` and
[Cache Behavior Workflow](../operations/cache-behavior-workflow.md).

Expected outputs:

- warm reuse for unchanged independent stages;
- invalidation limited to the branch affected by changed input;
- integrity refusal for a corrupted exact entry;
- a specific `why-cache-missed` identity or integrity reason.

`why-cache-missed` is a tested experimental explicit-path route, not a stable
root-help command.

## Failure Demo

Use `evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json` and
[Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md).

Expected outputs:

- attempt evidence for the transient fetch failure;
- a visible failed approval boundary;
- replay limited to the repaired boundary and descendants;
- strict verification of the repaired run.

## Replay Demo

Use the retained source from the
[First-Run Tutorial](../operations/first-run-tutorial.md):

```bash
bijux-dag replay --json --source-run-id first-run-tutorial-cold \
  --source-run-root ./artifacts/first-run-tutorial-runs \
  --out ./artifacts/first-run-tutorial-runs \
  --run-id first-run-tutorial-replay \
  --from-node render_report
```

Expected outputs:

- `replay_proof` in the response;
- a focused diff for `render_report`;
- retained source and parent run identity;
- strict verification success.

## Branch Demo

Use `evidence/dag/authoring/examples/audience-branch-bulletin.dag.json`:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/audience-branch-source/team-update.md"
bijux-dag run --json evidence/dag/authoring/examples/audience-branch-bulletin.dag.json \
  --out ./artifacts/audience-branch-runs \
  --run-id audience-branch-technical \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=technical"
```

Expected outputs:

- a retained branch decision;
- technical success and executive skip evidence;
- `selected_lane: technical` in `publish/selection.json`;
- stable branch selection under replay.

## Container Demo

Use `evidence/dag/authoring/examples/release-note-bundle.dag.json` with an
available supported container engine:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/release-note-source/weekly-update.md"
bijux-dag run --json evidence/dag/authoring/examples/release-note-bundle.dag.json \
  --out ./artifacts/release-note-bundle-runs \
  --run-id release-note-bundle \
  --input "source_note=${SOURCE_NOTE}" \
  --input "bundle_label=Release Brief"
```

Expected outputs:

- retained `bundle/release-note.txt`;
- retained `bundle/container-summary.json`;
- engine version and image identity in the node trace;
- an explicit infrastructure failure when no engine is available.

See [Container Packaging Workflow](../operations/container-packaging-workflow.md)
for mount, stream, and image-verification details.

## CI Recipe Contract

The test harness supplies isolated values for graph, run, cache, bundle, input,
and deliverable variables. Commands inside the markers below are executable
contract material: changing them requires running their owning integration
test, not only building the docs.

### Major Command Surface

This recipe covers stable execution and inspection plus the experimental
explicit-path `prove`, `export`, `import`, and `migrate inspect` routes.

<!-- recipe:ci-major-dag-commands:start -->
```bash
bijux-dag validate --json ${GRAPH}
bijux-dag plan explain --json ${GRAPH}
bijux-dag show-effective-graph --json ${GRAPH}
bijux-dag run --json ${GRAPH} --out ${RUN_ROOT} --run-id ${RUN_ID}
bijux-dag show-effective-graph --json --run-dir ${RUN_DIR}
bijux-dag explain --json ${RUN_DIR}
bijux-dag runs history --json --root ${RUN_ROOT} --status success --offset 0 --limit 5 --select run:${RUN_ID}
bijux-dag runs inspect ${RUN_ID} --root ${RUN_ROOT} --json
bijux-dag runs diagnostics-bundle ${RUN_ID} --root ${RUN_ROOT} --out ${DIAG_BUNDLE} --json --redact
bijux-dag runs index --root ${RUN_ROOT} --json
bijux-dag replay --json ${RUN_DIR} --out ${REPLAY_ROOT}
bijux-dag diff --json ${RUN_DIR} ${RUN_DIR}
bijux-dag prove --json ${RUN_DIR}
bijux-dag verify --json ${RUN_DIR}
bijux-dag export --json ${RUN_DIR} --out ${EXPORT_BUNDLE}
bijux-dag import --json --verify-only ${EXPORT_BUNDLE}
bijux-dag migrate inspect --json --run-dir ${RUN_DIR} --from v0.1 --to v0.1
```
<!-- recipe:ci-major-dag-commands:end -->

### Evidence-Backed Bulletin

This recipe proves cold and warm execution, changed-input comparison, focused
replay, strict verification, and artifact promotion.

<!-- recipe:ci-evidence-backed-bulletin:start -->
```bash
bijux-dag validate ${GRAPH}
bijux-dag run --json ${GRAPH} --out ${RUN_ROOT} --run-id branch-bulletin-cold --cache readwrite --cache-dir ${CACHE_ROOT} --input source_note=${SOURCE_NOTE} --input audience_mode=technical
bijux-dag run --json ${GRAPH} --out ${RUN_ROOT} --run-id branch-bulletin-warm --cache readwrite --cache-dir ${CACHE_ROOT} --input source_note=${SOURCE_NOTE} --input audience_mode=technical
bijux-dag artifact-inspect --json ${RUN_ROOT}/run-branch-bulletin-cold publish_bulletin:bulletin.md
bijux-dag artifact lineage ${RUN_ROOT}/run-branch-bulletin-cold --json
bijux-dag run --json ${GRAPH} --out ${RUN_ROOT} --run-id branch-bulletin-updated --cache readwrite --cache-dir ${CACHE_ROOT} --input source_note=${REVISED_NOTE} --input audience_mode=executive
bijux-dag runs compare branch-bulletin-warm branch-bulletin-updated --root ${RUN_ROOT} --json
bijux-dag run --json ${GRAPH} --out ${RUN_ROOT} --run-id branch-bulletin-proof-source --input source_note=${SOURCE_NOTE} --input audience_mode=executive
bijux-dag replay --json --source-run-id branch-bulletin-proof-source --source-run-root ${RUN_ROOT} --out ${RUN_ROOT} --run-id branch-bulletin-replay --select id:publish_bulletin --dependency-closure --prove
bijux-dag verify --json ${RUN_ROOT}/run-branch-bulletin-replay --strict
bijux-dag artifact promote ${RUN_ROOT}/run-branch-bulletin-updated publish_bulletin:bulletin.md --deliverables-root ${DELIVERABLES_ROOT} --to release --json
```
<!-- recipe:ci-evidence-backed-bulletin:end -->

### First-Run Proof

This is the executable form of the
[First-Run Tutorial](../operations/first-run-tutorial.md).

<!-- recipe:ci-first-run-tutorial:start -->
```bash
bijux-dag validate ${FILE_PROCESSING_GRAPH}
bijux-dag show-effective-graph --json ${FILE_PROCESSING_GRAPH}
bijux-dag run --json ${FILE_PROCESSING_GRAPH} --out ${RUN_ROOT} --run-id first-run-tutorial-cold --cache readwrite --cache-dir ${CACHE_ROOT} --input source_dir=${FILE_PROCESSING_SOURCE_DIR} --input report_title=First-Run-Tutorial-Report
bijux-dag explain ${RUN_ROOT}/run-first-run-tutorial-cold
bijux-dag artifact registry ${RUN_ROOT}/run-first-run-tutorial-cold --json
bijux-dag artifact-inspect --json ${RUN_ROOT}/run-first-run-tutorial-cold render_report:report.md
bijux-dag run --json ${FILE_PROCESSING_GRAPH} --out ${RUN_ROOT} --run-id first-run-tutorial-warm --cache readwrite --cache-dir ${CACHE_ROOT} --input source_dir=${FILE_PROCESSING_SOURCE_DIR} --input report_title=First-Run-Tutorial-Report
bijux-dag replay --json --source-run-id first-run-tutorial-cold --source-run-root ${RUN_ROOT} --out ${RUN_ROOT} --run-id first-run-tutorial-replay --from-node render_report
bijux-dag verify --json ${RUN_ROOT}/run-first-run-tutorial-replay --strict
```
<!-- recipe:ci-first-run-tutorial:end -->

## Next Reads

- [Operator Workflows](operator-workflows.md)
- [Run Evidence Layout](run-evidence-layout.md)
- [First-Run Tutorial](../operations/first-run-tutorial.md)
- [Cache Behavior Workflow](../operations/cache-behavior-workflow.md)
