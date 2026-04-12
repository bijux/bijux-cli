---
title: Environment Model
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Environment Model

The make layer centralizes environment defaults so local shells and CI jobs do
not quietly diverge.

## Important Defaults

- `VENV=artifacts/python/.venv` for the repo-managed Python environment
- `CARGO_TARGET_DIR` under `artifacts/` for Rust build outputs
- docs caches and site output under `artifacts/docs/`
- release and coverage outputs under `artifacts/`

## Environment Rule

Repository targets should default to artifact-scoped paths. If a target writes
outside `artifacts/`, that choice needs explicit justification.

## Next Reads

- [Repository Layout](repository-layout.md)
- [Package Contracts](package-contracts.md)
- [Artifact Governance](../../bijux-core/operations/artifact-governance.md)
