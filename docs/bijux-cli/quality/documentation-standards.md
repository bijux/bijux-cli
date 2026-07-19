---
title: Documentation Standards
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Documentation Standards

CLI documentation is treated as part of the contract surface, not optional
after-the-fact commentary.

Use this page when a code change affects what readers, script authors, or
operators will believe about the CLI and you need the rule for when the
handbook is good enough to publish.

## Standards

- every page must include frontmatter with owner and last review date
- every page must include concrete code anchors
- page claims must map to currently shipped behavior
- cross-links should point to canonical handbook pages only

## CLI Handbook Shape Standard

- package root index plus five section directories
- ten pages in each section
- no nested section trees under `docs/bijux-cli/`

## Migration Coverage Standard

Legacy chapter themes remain represented through current pages:

- introduction and getting-started material maps into foundation and operations
- reference and contracts material maps into interfaces and quality
- development and architecture material maps into architecture and quality

## What Good CLI Documentation Should Do

| Standard | Why readers need it |
| --- | --- |
| behavior-backed claims | users should not learn features the binary does not actually provide |
| concrete code anchors | maintainers should be able to audit where the claim comes from |
| canonical cross-links | readers should not get sent through stale or duplicate narratives |
| governed section shape | the handbook should stay navigable as the product grows |

## Code Anchors

- `mkdocs.yml`
- `makes/docs.mk`
- `docs/bijux-cli/`

## Reader Shortcut

If a page sounds polished but cannot be traced back to current behavior, tests,
or owned code, it is marketing language, not product documentation. This
handbook should prefer truth over polish every time.

## Continue Reading

- [Review Checklist](review-checklist.md)
- [Change Principles](../foundation/change-principles.md)
- [Release and Versioning](../operations/release-and-versioning.md)
