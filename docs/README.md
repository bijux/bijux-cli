# Documentation

This directory contains all project documentation.

## Structure
- `docs/spec/` — Formal specs (DAG, artifacts, validation, schemas)
- `docs/architecture/` — Architectural notes and diagrams
- `docs/adr/` — Architecture Decision Records (one per decision)
- `docs/operations/` — How to run, replay, diff, caching, failure semantics
- `docs/ADAPTERS.md` — Adapter API and examples
- `docs/POLICY.md` — Policy gates and effects enforcement

## Conventions
- Specs are versioned files (e.g., `*_v0.1.md`).
- ADRs are named `YYYYMMDD-title.md`.
- Build artifacts are written under `artifacts/` (local cache and generated targets).
- `artifacts/` is not committed; generated files belong under `artifacts/` and are ignored by default via `artifacts/.gitignore`.
- Build environment, tools, and cache policy is documented in `DEVELOPMENT.md`.
