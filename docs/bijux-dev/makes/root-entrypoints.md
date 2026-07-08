---
title: Root Entrypoints
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-05
---

# Root Entrypoints

Use this page when you want to know which root `make` targets matter first and
what kind of workflow each one is meant to cover.

The root make surface should be the first place a maintainer looks for a common
workflow. If a task matters often enough, the repository should either expose a
root target for it or make the package-local reason obvious.

## Primary Targets

- `make help` for the supported target catalog
- `make install` and `make bootstrap` for local setup
- `make fmt`, `make lint`, `make security`, `make test`, and `make build` for
  aggregate quality lanes
- `make test-all-frozen`, `make lint-frozen`, and `make audit-frozen` for
  detached pinned-commit verification runs
- `make docs-check` for handbook integrity
- `make dag-help` for DAG governance entrypoints

## Choose The Right Starting Target

| Situation | First target |
| --- | --- |
| you need to discover what the repository exposes | `make help` |
| you are preparing a machine or checkout | `make install` or `make bootstrap` |
| you are checking ordinary code health | `make fmt`, `make lint`, `make test`, `make build` |
| you are validating handbook integrity | `make docs-check` |
| you are working on detached frozen verification | `make test-all-frozen`, `make lint-frozen`, or `make audit-frozen` |
| you are entering DAG-specific maintainer flows | `make dag-help` |

## Entrypoint Rule

If a repeated workflow matters, it should have a root target or a documented
reason why it stays package-local.

## What This Page Is Not Saying

- It is not listing every target in the repository.
- It is not replacing `make help` for the full live catalog.
- It is not saying package-local workflows are bad when their scope is truly
  local.

## Continue Reading

- [Environment Model](environment-model.md)
- [CI Targets](ci-targets.md)
- [Release Surfaces](release-surfaces.md)
