---
title: Make Target Authoring
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Make Target Authoring

A root Make target is a repository interface. It must preserve the underlying
command's selection, output, and failure status while making ownership easier
to discover.

## Existing Execution Contract

The shared environment configures:

```make
SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
```

Recipes therefore fail on unset variables, failed commands, and failed
pipeline members. Do not weaken those defaults inside a target. A recipe that
tees output must still return the underlying command status.

Each recipe line runs in a separate shell. Keep state on one continued recipe
or pass it explicitly; do not assume `cd`, shell variables, traps, or exported
values survive into the next line.

## Ownership And Placement

| Behavior | Owner |
| --- | --- |
| organization-wide target composition, guards, or Rust lane behavior | `bijux-std`, consumed under `.bijux/shared/` |
| repository bootstrap and aggregate entrypoints | `makes/_internal.mk` |
| Rust, Python, docs, GitHub, DAG, or standards orchestration | matching local fragment under `makes/` |
| product semantics | owning crate or Python package |
| workflow trigger and hosted permissions | owning GitHub workflow |

Do not hand-edit `.bijux/shared/`. Parameterize a documented shared variable
locally, or repair the shared source in `bijux-std` and refresh from an accepted
commit.

## Target Shape

A stable target should:

- be declared `.PHONY`;
- use a durable concern-based name;
- include a concise `##` help description;
- expose caller configuration through documented `?=` variables;
- keep fixed safety boundaries as immediate or simply expanded assignments;
- invoke product or maintainer code rather than duplicating policy in shell;
- write generated output below `$(ARTIFACT_ROOT_ABS)`;
- print enough command or artifact context to diagnose a failure.

Use `$(MAKE) --no-print-directory` or the shared `$(BIJUX_MAKE)` value for
recursive invocation. Directly invoking `make` can lose jobserver behavior,
flags, and caller overrides.

## Failure Preservation

### Pipelines

The repository shell already has `pipefail`, but status can still be lost if a
recipe disables error handling or appends unconditional success. Prefer:

```make
report:
	@mkdir -p "$(REPORT_DIR)"
	@command-that-must-pass 2>&1 | tee "$(REPORT_DIR)/report.log"
```

When several commands must all run even after one fails, capture each status,
print a final summary, and return nonzero if any component failed. Do not use
`|| true` to keep going unless the failure is explicitly best-effort and the
final result still reports it.

### Aggregate Targets

Ordinary prerequisite chains may stop when one prerequisite fails. They are
appropriate when later work depends on earlier success. They are not
appropriate when the contract requires every independent component to run and
a complete final summary.

For complete test portfolios, delegate to the governed runner that aggregates
test status, such as nextest through `make test-all`. Do not build an
apparently complete lane from prerequisites that silently short-circuit on the
first failure.

### Background Work

A printed PID proves only that work started. A background target must publish:

- immutable source identity;
- console and artifact paths;
- PID or process metadata;
- terminal status file;
- final tool summary.

The launcher must not return a fabricated pass while the terminal result is
unknown.

## Artifact Discipline

Use repository variables rather than ad hoc paths:

- Rust: `$(RS_ARTIFACT_ROOT)` and `$(RS_TARGET_DIR)`;
- Python: managed environment and package artifact variables;
- docs: `artifacts/docs/`;
- frozen gates: commit-keyed roots under `artifacts/`;
- one-off local output: a named subtree under `$(ARTIFACT_ROOT_ABS)`.

Tracked generated files under `docs/spec`, `docs/reports`, or another governed
path require a named producer and validation contract. They are not ordinary
run output.

Cleanup must stay inside the artifact boundary and use the shared safe-removal
guard where available. A caller override must not turn `clean` into arbitrary
filesystem deletion.

## Review Checklist

- Does `make help` expose the target when it is intended for routine use?
- Does the fragment match the behavior owner?
- Are shared managed files untouched?
- Does every pipeline and wrapper preserve nonzero status?
- Does an aggregate run all components its name claims?
- Are all generated paths inside the artifact or governed-output boundary?
- Are caller overrides intentional and validated?
- Can a maintainer identify the underlying command and retained evidence?
- Is the narrowest relevant target tested before commit?

## Related Guidance

- [Make Execution Model](make-system-overview.md)
- [Package Dispatch](package-dispatch.md)
- [CI Targets](ci-targets.md)
- [Artifact Governance](../../bijux-core/operations/artifact-governance.md)
