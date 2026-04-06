---
title: Documentation Standard
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Documentation Standard

Maintainer documentation must be practical, testable, and consistent with the
shared handbook format used across Bijux repositories.

## Visual Summary

```mermaid
flowchart LR
    structure[section structure contract] --> style[language and tone consistency]
    style --> anchors[code anchors and examples]
    anchors --> validation[docs-check validation]
```

## Standards

- use canonical frontmatter on every page
- keep links within active handbook tree
- include one diagram per page for system relationships
- connect policies to real commands, files, or tests
- avoid placeholder language without operational meaning

## Language and Tone Rules

- use imperative or declarative policy language for required behavior
- avoid hand-wavy phrasing that cannot be validated by commands or files
- prefer short sentences that map directly to maintainer actions
- keep remediation guidance concrete and ordered when incidents are involved

## Required Alignment

- `docs/bijux-core` and `docs/bijux-dev` follow parallel section patterns
- maintainer docs avoid duplicating CLI and DAG product semantics
- MkDocs nav remains synchronized with filesystem layout

## Canonical Maintainer Page Template

````md
---
title: <Page Title>
audience: maintainers
type: <operations|governance|section-index>
status: canonical
owner: bijux-dev-docs
last_reviewed: YYYY-MM-DD
---

# <Page Title>

## Visual Summary
```mermaid
flowchart TD
    A --> B
```

## Operational Rules

## Code Anchors

## Next Reads
````

## Code Anchors

- `mkdocs.yml`
- `docs/bijux-dev/`
- `makes/docs.mk`

## Next Reads

- [Docs Operations](../operations/docs-operations.md)
- [Core Documentation Standards](../../bijux-core/governance/documentation-standards.md)
- [Known Limitations](known-limitations.md)
