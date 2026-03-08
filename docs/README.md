# Documentation

Audience: all documentation consumers.  
Owner: documentation maintainers.  
Status: stable.

This directory is the top-level documentation area for the project.

## Source of truth

Global documentation policy is defined in [docs-information-architecture.md](../docs-information-architecture.md).
All structural cleanup, consolidation, and retention decisions for docs are guided by this document.

## Primary entrypoints

- [`index.md`](./index.md) — discovery path and curated navigation
- [`README.md`](./README.md) — this file

## Allowed top-level documentation sections

- `architecture/` — live system maps and boundary documentation
- `adr/` — durable historical decisions
- `operations/` — maintainer operation workflows
- `reference/` — operator and maintainer reference material
- `testing/` — maintainer and contributor testing workflows
- `dev/` — contributor and local development workflows
- `reports/` — curated human-readable summaries
- `spec/` — canonical contracts
- `user/` — beginner and operator guides
- `generated/` — committed generated outputs that are intentionally retained

No other top-level folders in `docs/` should be added without updating the master architecture file.

## Section conventions

- Root-level docs are entrypoints only. They should explain where to start, not detailed implementation behavior.
- Every documentation file must belong to one audience, one owner, and one status: `stable`, `generated`, `historical`, or `internal`.
- Generated output must be separated from hand-authored guidance.
- Reference and spec content must avoid duplication of source-of-truth contracts.

## Governance automation

- `make docs-governance-lint` checks metadata, duplicate titles/topics, and orphan-document signals.
- `make docs-inventory-generate` regenerates:
  - `docs/generated/DOCS_INVENTORY.md`
  - `docs/generated/DOCS_CONSOLIDATION_CANDIDATES.md`
