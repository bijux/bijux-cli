---
title: Documentation Standards
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-05
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

## Limitation Records

Known limitations are release-facing records, not generic caution prose. When a
page documents a live DAG limitation, each record must include:

- a stable limitation id
- a stability class such as `stable-surface`, `experimental-surface`, or `simulation-surface`
- the affected command, API, or namespace
- the actual limitation
- operator impact
- a concrete workaround
- the planned fix direction
- the release target or explicit statement that no guarantee exists in the
  current release line

Use this record shape for `known-limitations.md` so operators can tell what they
must not rely on and maintainers can verify whether a limitation has really
changed.

## Legacy Mapping Policy

Legacy nested DAG chapters are intentionally consolidated into the five canonical
sections: `foundation`, `architecture`, `interfaces`, `operations`, and
`quality`.

## Next Reads

- [Review Checklist](review-checklist.md)
- [Definition of Done](definition-of-done.md)
- [DAG Documentation Index](../index.md)
