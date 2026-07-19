---
title: Executable Examples
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Executable Examples

This page is the public, executable examples authority for `bijux-dag`
v0.4.0.

Use it when the question is "which repository example should I run for this
behavior, and what should I expect to see when it works?"

Every example listed here is backed by a checked-in graph, a repository-owned
fixture set, or a repository-tested guide. The index stays on the honest
release boundary: local-first execution, retained evidence, cache and replay
proof, branch visibility, and container packaging when the required engine is
present.

The recipe blocks near the end of this page are executed by
`crates/bijux-dag-app/tests/docs_executable_recipes_contract.rs`. Stable
commands remain on the visible `bijux-dag --help` surface. Experimental
commands are experimental explicit-path routes, identified rather than
presented as stable. The
[Release Boundary](../foundation/release-boundary.md) and
`contracts/foundation/dag_release_truth_table.v1.json` govern those
classifications.

## Example Map

| Example | Primary surface | Graph or guide | Expected outputs |
| --- | --- | --- | --- |
| minimal hello DAG | validate and run a tiny graph | `evidence/dag/authoring/examples/hello.dag.json` | retained run directory, constant output, shell output |
| file-processing DAG | run one practical local report workflow | `evidence/dag/authoring/examples/file-processing-report.dag.json` | rendered report, artifact registry entry, replayable run per [Replay Contract](../../spec/REPLAY_CONTRACT.md) |
| cache demo | prove warm reuse and selective invalidation | [Cache Behavior Workflow](../operations/cache-behavior-workflow.md) | warm cache hits, changed-input invalidation, cache-miss explanation |
| failure demo | prove retry evidence and focused repair | [Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md) | retry attempt record, failed approval boundary, repaired verified run |
| replay demo | rerun a selected boundary from retained evidence | [First-Run Tutorial](../operations/first-run-tutorial.md) | replay proof, node-scoped rerun diff, strict verification success |
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
[File Processing Workflow](../operations/file-processing-workflow.md)

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
[Cache Behavior Workflow](../operations/cache-behavior-workflow.md)

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
[Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md)

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
[First-Run Tutorial](../operations/first-run-tutorial.md)

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
[Branching Bulletin Workflow](../operations/branching-bulletin-workflow.md)

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
[Container Packaging Workflow](../operations/container-packaging-workflow.md)

## CI-Executed Recipes

The recipes use variables supplied by the test harness:

- `${GRAPH}`, `${FILE_PROCESSING_GRAPH}`, and
  `${FILE_PROCESSING_SOURCE_DIR}` identify checked-in fixtures
- `${RUN_ROOT}`, `${RUN_ID}`, and `${RUN_DIR}` identify retained runs
- `${REPLAY_ROOT}`, `${EXPORT_BUNDLE}`, and `${DIAG_BUNDLE}` identify evidence
  outputs
- `${SOURCE_NOTE}` and `${REVISED_NOTE}` identify bulletin inputs
- `${CACHE_ROOT}` and `${DELIVERABLES_ROOT}` isolate cache and promoted output

### Major Command Surface

This recipe covers the stable execution and inspection flow plus the
experimental explicit-path `prove`, `export`, `import`, and `migrate inspect`
routes.

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

This recipe proves cold and warm execution, retained artifact inspection,
changed-input comparison, focused replay, strict verification, and promotion.

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

This is the compact executable form of the
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

- [Entrypoints and Examples](entrypoints-and-examples.md)
- [Operator Workflows](operator-workflows.md)
- [First-Run Tutorial](../operations/first-run-tutorial.md)
- [Cache Behavior Workflow](../operations/cache-behavior-workflow.md)
- [Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md)
