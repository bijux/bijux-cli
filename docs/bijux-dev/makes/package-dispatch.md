---
title: Make Dispatch Boundaries
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Make Dispatch Boundaries

Make owns orchestration, not product behavior. Use this page to trace a failed
root target to the command, package, and evidence surface that can actually
explain it.

## Dispatch Map

| Root family | Immediate adapter | Behavioral owner | Typical evidence |
| --- | --- | --- | --- |
| `fmt`, `lint`, Rust tests, audit, coverage, Rust docs | shared Rust gate plus `makes/rust.mk` parameters | Cargo workspace and focused crate tests | `artifacts/rust/` |
| Python test, lint, security, build, publish | `makes/python.mk` | `crates/bijux-cli-python` and native `bijux-cli` bridge | `artifacts/python/` |
| documentation checks and serving | `makes/docs.mk`, `makes/bijux-docs.mk` | Markdown, MkDocs configuration, and docs automation | `artifacts/docs/` |
| DAG governance and evidence | `makes/dag.mk` | `bijux-dev-dag` commands and owning product contracts | named DAG or report artifact roots |
| GitHub entrypoints | `makes/gh.mk` | delegated local target; workflow owns hosted setup | workflow log plus repository artifacts |
| standards validation and refresh | `makes/bijux-std.mk` | accepted `bijux-std` source and sync tooling | checksum and refresh report |

The adapter is the first debugging location only when it changed selection,
environment, status, or paths. A product assertion failure belongs to the
owning crate even when Make launched it.

## Trace A Failure

1. Read the exact command printed by the target and its final exit status.
2. Identify the local fragment and any shared target it delegates to.
3. Inspect caller overrides and effective environment.
4. Open the retained console or report under `artifacts/`.
5. Reproduce the smallest owning command without changing selection.
6. Fix the layer that introduced the defect.

Do not copy a product command into a workflow to avoid a broken Make target.
That creates two orchestration contracts and hides the local failure.

## Boundary Tests

A dispatch wrapper must preserve:

- source commit and worktree assumptions;
- test or suite selection;
- toolchain and feature flags;
- stdout, stderr, and structured report locations;
- terminal status and complete summary;
- artifact containment.

If the wrapper adds retries, filtering, advisory mode, environment variables,
or ignored-test behavior, that behavior is part of the wrapper contract and
requires focused coverage.

## Related Guidance

- [Make Execution Model](make-system-overview.md)
- [Make Target Authoring](authoring-rules.md)
- [Root Entrypoints](root-entrypoints.md)
- [Repository Gates](../operations/repository-gates.md)
