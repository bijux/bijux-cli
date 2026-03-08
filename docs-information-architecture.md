# Documentation information architecture

**Date:** 2026-03-09
**Status:** Active source of truth for documentation layout, ownership, and cleanup decisions.

## Scope

This document is the migration source of truth for documentation structure and consolidation decisions in `docs/`.
All future documentation cleanup proposals must match the rules in this file.

## Final top-level sections

Only the following top-level sections are allowed in the final tree:

- `docs/README.md` and `docs/index.md` (entrypoints only)
- `docs/user/`
- `docs/reference/`
- `docs/spec/`
- `docs/architecture/`
- `docs/adr/`
- `docs/operations/`
- `docs/dev/`
- `docs/testing/`
- `docs/reports/`
- `docs/generated/`

No new ad hoc top-level folders under `docs/` may be added for documentation content.
Temporary folders used for migration are not included in this permission list.

## Section roles

### `docs/reports/`
Primary role is curated maintainer evidence and historical context. It is **not** a first-class discovery path.
Generated reports must move to artifacts or equivalent build output.

### `docs/spec/`
Canonical contract library only.
No explanatory duplicates, tutorials, or roadmap narratives.
Each spec file must state contract purpose and stable API-style intent.

### `docs/reference/`
Operator and maintainer reference material only.
Includes command catalogs, support matrices, indexes, and schemas.

### `docs/architecture/`
Living architecture boundaries, runtime maps, and integration boundary documentation.
No historical rationale records.

### `docs/adr/`
Durable historical decisions only.
ADRs are immutable records; no user or runbook content.

### `docs/user/`
Beginner and operator guides only.
One path should cover onboarding, installation, and common operations.

### `docs/operations/`
Maintainer-facing implementation operations and practical execution workflows.

### `docs/dev/`
Developer workflows, contribution practices, and local tooling guidance.

### `docs/testing/`
Maintainer and contributor test execution workflows.

Decision: `dev`, `operations`, and `testing` remain separate for now to preserve clear audience focus.

### `docs/README.md` and `docs/index.md`
Root entrypoint docs.
Only high-signal orientation docs.

### `docs/generated/`
Build-time artifacts only.
No hand-authored documentation.

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

- Archive all old planning and simulation narratives that do not support one of: use, guarantee, structure, durable decision, or curated evidence.
- Merge duplicates into the nearest canonical section.
- Move stale generated output to non-doc artifact locations.
- Keep the most recent relevant report per domain in `docs/reports/` when it directly supports operator action.
- Reject new top-level docs additions that cannot be mapped to this architecture in one of the allowed sections above.

## Change control freeze

- The information architecture is frozen after this migration.
- Any new document must include:
  - audience
  - owner
  - status (`stable`, `generated`, `historical`, or `internal`)
  - explicit link from an index or a declared `Standalone: yes` marker
- New top-level folders under `docs/` require an explicit architecture update in this file within the same change.
