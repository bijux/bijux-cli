---
title: Definition of Done
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Definition of Done

Use this page when a change looks finished in code review but you need the
harder answer: is it actually ready to leave review and become part of the
trusted CLI surface?

The line between "implemented" and "done" matters here. A `bijux-cli` change
is only done when behavior, evidence, and the written contract all point at the
same result.

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

## What “Done” Must Mean

| Surface | What reviewers should be able to trust |
| --- | --- |
| code | the owning implementation actually carries the intended behavior |
| tests | the changed contract is exercised by the correct lane |
| docs | readers can learn the new behavior without guessing |
| compatibility framing | downstream consumers know whether adaptation is required |

## Not Done Signals

- docs still describe old behavior
- tests are missing for contract-impacting changes
- compatibility risk is implied but not stated
- review checklist items are skipped without rationale

## Reader Shortcut

If the team needs verbal context from the author to explain why a change is
safe, the change is not done yet. The repository itself should carry enough
evidence for a new reviewer to reach the same conclusion.

## Continue Reading

- [Review Checklist](review-checklist.md)
- [Change Validation](change-validation.md)
- [Release and Versioning](../operations/release-and-versioning.md)
