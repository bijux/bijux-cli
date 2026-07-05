---
title: Executable Recipes
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-30
---

# Executable Recipes

This page defines one deterministic command recipe for major DAG operator
surfaces. The recipe is also executed by CI from
`crates/bijux-dag-app/tests/docs_executable_recipes_contract.rs`.

The stable examples on this page stay on the visible `bijux-dag --help`
surface. When a recipe intentionally reaches into explicit-path experimental
routes, the text calls that out rather than implying stable support.

## Variables

- `${GRAPH}`: deterministic graph fixture path
- `${RUN_ROOT}`: run root directory
- `${RUN_ID}`: primary run identifier
- `${RUN_DIR}`: `${RUN_ROOT}/${RUN_ID}`
- `${REPLAY_ROOT}`: replay output root directory
- `${EXPORT_BUNDLE}`: exported replay bundle path
- `${DIAG_BUNDLE}`: exported diagnostics bundle path

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
bijux-dag run --json ${GRAPH} --out ${RUN_ROOT} --run-id ${RUN_ID}
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
