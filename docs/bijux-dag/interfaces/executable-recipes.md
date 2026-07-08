---
title: Executable Recipes
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Executable Recipes

This page defines one deterministic command recipe for major DAG operator
surfaces. The recipe is also executed by CI from
`crates/bijux-dag-app/tests/docs_executable_recipes_contract.rs`.

The stable examples on this page stay on the visible `bijux-dag --help`
surface. When a recipe intentionally reaches into explicit-path experimental
routes, the text calls that out rather than implying stable support.

The release-boundary source of truth for those classifications is the
[Release Boundary](../foundation/release-boundary.md) page, which is backed by
the machine-readable contract
`contracts/foundation/dag_release_truth_table.v1.json`.

## Variables

- `${GRAPH}`: deterministic graph fixture path
- `${RUN_ROOT}`: run root directory
- `${RUN_ID}`: primary run identifier
- `${RUN_DIR}`: `${RUN_ROOT}/${RUN_ID}`
- `${REPLAY_ROOT}`: replay output root directory
- `${EXPORT_BUNDLE}`: exported replay bundle path
- `${DIAG_BUNDLE}`: exported diagnostics bundle path
- `${SOURCE_NOTE}`: original bulletin source note
- `${REVISED_NOTE}`: revised bulletin source note
- `${CACHE_ROOT}`: cache root directory
- `${DELIVERABLES_ROOT}`: deliverables root directory
- `${FILE_PROCESSING_GRAPH}`: file-processing graph fixture path
- `${FILE_PROCESSING_SOURCE_DIR}`: file-processing source directory

## CI Recipe: Major DAG Commands

This recipe intentionally spans two lanes:

- stable operator surface: `validate`, `plan explain`, `run`, `runs ...`,
  `replay`, `diff`, and `verify`
- experimental explicit-path routes: `prove`, `export`, `import`, and
  `migrate inspect`

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

## CI Recipe: Evidence-Backed Bulletin Workflow

This recipe executes the complete retained bulletin workflow described in
[Evidence-Backed Bulletin Workflow](../operations/guides/evidence-backed-bulletin-workflow.md).
It stays on the stable operator surface except for `artifact-inspect`, which is
part of the stable visible command inventory for retained artifact inspection.

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

## CI Recipe: First-Run Tutorial

This recipe executes the compact onboarding path described in
[First-Run Tutorial](../operations/guides/first-run-tutorial.md). It uses the
stable visible operator surface for execution, retained artifacts, warm cache
reuse, focused replay, and strict verification, plus the explicit-path
`show-effective-graph` route for pre-execution graph inspection.

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
