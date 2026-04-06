---
title: Documentation Standards
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Documentation Standards

Documentation standards keep this repository aligned with the shared Bijux
handbook model and reduce cognitive switching between projects.

## Visual Summary

```mermaid
flowchart LR
    structure[stable structure] --> navigation[predictable navigation]
    navigation --> language[consistent language and tone]
    language --> trust[reader trust]
    trust --> maintainability[maintainer maintainability]
```

## Standards

- each handbook keeps a stable top-level section architecture
- every page includes canonical frontmatter keys
- behavior claims link to code anchors or executable checks
- examples use current command names and realistic paths
- diagrams summarize key system relationships on each page

## Required Frontmatter Keys

- `title`
- `audience`
- `type`
- `status`
- `owner`
- `last_reviewed`

## Code Anchors

- `mkdocs.yml`
- `mkdocs.shared.yml`
- `makes/docs.mk`
- `docs/index.md`

## Next Reads

- [Decision Record Policy](decision-record-policy.md)
- [Risk and Exceptions](risk-and-exceptions.md)
- [Maintainer Documentation Standard](../../bijux-dev/governance/documentation-standard.md)
