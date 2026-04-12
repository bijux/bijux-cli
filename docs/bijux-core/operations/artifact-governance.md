---
title: Artifact Governance
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Artifact Governance

Generated outputs are useful only when they stay auditable and do not leak into
tracked source roots by accident.

## Repository Rules

- generated outputs belong under `artifacts/`
- documentation builds must not create tracked `site/` or `.cache/` roots
- release bundles should be reproducible from tagged source and release-tree
  preparation

## Examples

- docs output under `artifacts/docs/site`
- Python build output under `artifacts/python/build`
- Rust test, lint, and coverage reports under `artifacts/rust/`

## Code Anchors

- `makes/docs.mk`
- `makes/python.mk`
- `makes/rust.mk`

## Next Reads

- [Release and Versioning](release-and-versioning.md)
- [Automation Surfaces](automation-surfaces.md)
- [Maintainer Docs Operations](../../bijux-dev/operations/docs-operations.md)
