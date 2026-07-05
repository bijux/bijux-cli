---
title: Root Entrypoints
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-05
---

# Root Entrypoints

The root make surface should be the first place a maintainer looks for a common
workflow.

## Primary Targets

- `make help` for the supported target catalog
- `make install` and `make bootstrap` for local setup
- `make fmt`, `make lint`, `make security`, `make test`, and `make build` for
  aggregate quality lanes
- `make test-all-frozen`, `make lint-frozen`, and `make audit-frozen` for
  detached pinned-commit verification runs
- `make docs-check` for handbook integrity
- `make dag-help` for DAG governance entrypoints

## Entrypoint Rule

If a repeated workflow matters, it should have a root target or a documented
reason why it stays package-local.

## Next Reads

- [Environment Model](environment-model.md)
- [CI Targets](ci-targets.md)
- [Release Surfaces](release-surfaces.md)
