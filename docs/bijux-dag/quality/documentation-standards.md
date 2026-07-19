---
title: Documentation Standards
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
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

- every handbook page has canonical frontmatter and a clear audience;
- examples use current command names, explicit inputs, and artifact-scoped
  paths;
- links and anchors resolve in source and in the curated public build;
- source anchors point to real owning crates, modules, contracts, or commands;
- diagrams are used only when they communicate structure more clearly than
  prose or a table;
- generated references name their producer and are not hand-edited;
- public claims stay within the release boundary and name limitations.

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

## Risk Records

The DAG risk register must also be operational rather than thematic. Each live
risk record in `risk-register.md` must include:

- a stable risk id
- severity
- the affected component, release surface, or command area
- the current status
- the actual risk
- the mitigation or monitoring action
- the release decision attached to that risk

Use this record shape when a DAG release concern could block, narrow, or
condition operator trust. The point is to make release posture reviewable
without asking maintainers to infer the real decision from vague prose.

## Handbook Shape

The DAG handbook has six durable sections: `foundation`, `architecture`,
`interfaces`, `operations`, `quality`, and `packages`. Pages stay directly
under their owning section. Section sizes are driven by distinct reader
questions, not a fixed page quota.

`docs/spec` remains the executable-contract layer and `docs/reports` remains
the governed evidence layer. They are repository inputs, not extra public
handbook sections.

## Next Reads

- [Review Checklist](review-checklist.md)
- [Definition of Done](definition-of-done.md)
- [DAG Documentation Index](../index.md)
