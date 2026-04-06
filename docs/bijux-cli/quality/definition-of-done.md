---
title: Definition of Done
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Definition of Done

A `bijux-cli` change is done only when behavior, tests, and documentation are
all aligned and reviewable.

## Visual Summary

```mermaid
flowchart LR
    implementation["implementation complete"] --> tests["targeted tests pass"]
    tests --> docs["handbook and diagrams updated"]
    docs --> review["compatibility and boundary review complete"]
    review --> done["ready for release"]
```

## Done Criteria

- behavior change is implemented in the owning module
- relevant routing/integration/architecture tests are updated and passing
- affected handbook pages are updated with concrete code anchors and diagrams
- compatibility impact is explicitly documented when contract-facing
- no unresolved blocking risk remains in the review thread

## Code Anchors

- `crates/bijux-cli/src/`
- `crates/bijux-cli/tests/`
- `docs/bijux-cli/`
- `makes/docs.mk`

## Not Done Signals

- docs still describe old behavior
- tests are missing for contract-impacting changes
- compatibility risk is implied but not stated
- review checklist items are skipped without rationale

## Next Reads

- [Review Checklist](review-checklist.md)
- [Change Validation](change-validation.md)
- [Release and Versioning](../operations/release-and-versioning.md)
