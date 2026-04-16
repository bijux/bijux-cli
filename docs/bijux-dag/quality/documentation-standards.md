---
title: Documentation Standards
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Documentation Standards

DAG documentation must remain executable as guidance: accurate, linked, and
aligned with real code behavior.

## Visual Summary

```mermaid
flowchart LR
  Page[Documentation page] --> Purpose[Clear purpose]
  Page --> Anchors[Concrete code and workflow anchors]
  Page --> Scope[Honest ownership scope]
  Page --> Diagram[Useful diagram fit]
  Page --> Examples[Examples where needed]
  Page --> Review[Review metadata current]
```

## Standards

- every page has canonical frontmatter and clear audience
- examples use current command names and realistic paths
- links remain within current docs tree and avoid removed paths
- code anchors point to real crate/module surfaces
- mermaid diagrams summarize core relationships on each page

## Legacy Mapping Policy

Legacy nested DAG chapters are intentionally consolidated into the five canonical
sections: `foundation`, `architecture`, `interfaces`, `operations`, and
`quality`.

## Next Reads

- [Review Checklist](review-checklist.md)
- [Definition of Done](definition-of-done.md)
- [DAG Documentation Index](../index.md)
