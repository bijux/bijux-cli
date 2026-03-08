# Documentation

Audience: all documentation consumers.  
Owner: documentation maintainers.  
Status: stable.

This directory is the top-level documentation area for the project and the canonical source of truth for documentation layout, ownership, and governance.

## Source of truth

Global documentation policy is defined in this file.

## Primary entrypoints

- [`index.md`](./index.md) — discovery path and curated navigation
- [`README.md`](./README.md) — this file

## Documentation Information Architecture

**Date:** 2026-03-09  
**Status:** Active source of truth for documentation layout, ownership, and cleanup decisions.

## Final top-level sections

- `docs/README.md` and `docs/index.md` (entrypoints only)
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

No new ad hoc top-level folders under `docs/` may be added for documentation content.

## Section roles

- `docs/reports/`: curated maintainer evidence and historical context, not a first-class discovery path.
- `docs/spec/`: canonical contract library only, no explanatory duplicates or tutorials.
- `docs/reference/`: operator and maintainer reference material only.
- `docs/architecture/`: living architecture boundaries and runtime maps.
- `docs/adr/`: durable historical decisions only.
- `docs/user/`: beginner and operator guides only.
- `docs/operations/`: maintainer-facing execution workflows.
- `docs/dev/`: development and contribution workflows.
- `docs/testing/`: maintainer and contributor test workflows.
- `docs/generated/`: build-time artifacts only.

## Hard rules for every remaining doc

1. Root-level docs are entrypoints only and must answer one top-level question.
2. Every doc declares one primary audience.
3. Every doc declares one owner.
4. Every doc declares status as one of: `stable`, `generated`, `historical`, or `internal`.
5. Generated outputs must not be stored beside hand-authored guides.
6. No document may duplicate contract text already published in `docs/spec/`.
7. No doc may duplicate matrix/report content that already lives in generated outputs.

## Target counts

- Root-level docs (excluding README/index): **18**
- `docs/spec/`: **120**
- `docs/reference/`: **35**
- `docs/architecture/`: **20**
- `docs/adr/`: **22**
- `docs/user/`: **10**
- `docs/operations/`: **16**
- `docs/reports/`: **24** durable summaries in-tree

## Approved migration posture

- Archive old planning and simulation narratives that do not support use, guarantee, structure, durable decision, or curated evidence.
- Merge duplicates into the nearest canonical section.
- Move stale generated output to non-doc artifact locations.
- Keep only recent, relevant reports in `docs/reports/` when they directly support operator action.
- Reject new top-level docs additions that cannot be mapped to this architecture.

## Governance automation

- `make docs-governance-lint` checks metadata, duplicate titles/topics, and orphan-document signals.
- `make docs-inventory-generate` regenerates:
  - `docs/generated/DOCS_INVENTORY.md`
  - `docs/generated/DOCS_CONSOLIDATION_CANDIDATES.md`

## Change control freeze

- The information architecture is frozen after this migration.
- Any new document must include:
  - audience
  - owner
  - status (`stable`, `generated`, `historical`, or `internal`)
  - explicit link from an index or a declared `Standalone: yes` marker
- New top-level folders under `docs/` require an explicit architecture update in this file within the same change.
