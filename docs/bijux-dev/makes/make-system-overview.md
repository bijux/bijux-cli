---
title: Make Execution Model
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Make Execution Model

The root `Makefile` contains one include: `makes/root.mk`. That file is the
composition boundary between organization-wide Make policy and workflows owned
by this repository. Reading that include graph is the fastest way to determine
whether a failed target belongs in `bijux-std`, in a local adapter, or in the
underlying Rust or Python package.

## Composition Boundary

`makes/root.mk` loads the layers in an intentional order:

1. `_macro.mk` establishes local shell helpers and the artifact-root guard.
2. `.bijux/shared/bijux-makes/environment.mk` and `guards.mk` provide the
   synchronized cross-repository environment and safety contracts.
3. `_internal.mk` defines bootstrap, cleanup, aggregate targets, and the
   repository-managed Python environment.
4. Repository fragments define Rust, Python, documentation, standards,
   GitHub, and DAG workflows.
5. `.bijux/shared/bijux-makes-rs/bijux.mk` supplies the governed Rust test
   lanes and their reporting behavior.

The files under `.bijux/shared/` are generated standards content. A local
workflow may consume or parameterize those files, but must not hand-edit them.
Changes to shared behavior originate in `bijux-std`; changes specific to this
workspace belong in `makes/`.

## Ownership Map

| Surface | Owning file | Responsibility |
| --- | --- | --- |
| shell guardrails | `makes/_macro.mk` | local reusable checks and artifact-safe deletion |
| setup and aggregates | `makes/_internal.mk` | virtual environment, cleanup, and root quality targets |
| Rust workflows | `makes/rust.mk` | build, lint, security, coverage, and release validation |
| governed Rust tests | `.bijux/shared/bijux-makes-rs/bijux.mk` | fast, slow, complete, and frozen test lanes |
| Python workflows | `makes/python.mk` | bridge tests, packaging, and publication |
| handbook workflows | `makes/docs.mk`, `makes/bijux-docs.mk` | local site checks and shared documentation shell |
| standards refresh | `makes/bijux-std.mk` | synchronized governance content |
| hosted automation | `makes/gh.mk` | commands invoked by GitHub Actions |
| DAG maintenance | `makes/dag.mk` | DAG evidence and governance commands |

This map describes command ownership, not product ownership. A Make target may
orchestrate several packages, but the package implementing the behavior remains
the authority for product semantics.

## Environment And Outputs

Repository targets default generated state to `artifacts/`:

- `VENV=artifacts/python/.venv` contains the managed Python environment.
- Rust targets set `CARGO_TARGET_DIR` to an artifact-scoped directory.
- MkDocs site and cache data live under `artifacts/docs/`.
- coverage, release, frozen-run, and evidence outputs remain under their
  corresponding artifact subtrees.

`make env` prints the effective Python and runtime values. Rust and workflow
fragments expose additional variables near the targets that consume them.
Callers may override documented `?=` variables, but fixed repository invariants
such as the managed `VENV` are not ad hoc extension points.

## Placement Rules

- Put organization-wide behavior in `bijux-std`, then refresh the synchronized
  shared content.
- Put repository orchestration in the local fragment matching its concern.
- Keep product logic in the owning crate or package, not in shell recipes.
- Give a stable root target to a workflow that contributors or CI invoke
  routinely.
- Make failure output reveal the underlying tool or package.
- Default every generated output to `artifacts/` unless the output is a
  governed repository source.

A target in the wrong fragment is an ownership defect: documentation rules do
not belong in `gh.mk` merely because CI calls them, and package release logic
does not belong in `_internal.mk` merely because it is broadly used.

## Related Guidance

- [Root Entrypoints](root-entrypoints.md)
- [Package Dispatch](package-dispatch.md)
- [Authoring Rules](authoring-rules.md)
- [Artifact Governance](../../bijux-core/operations/artifact-governance.md)
