---
title: DAG Handbook
audience: mixed
type: index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# DAG Handbook

`bijux-dag` owns DAG modeling, execution, replay, and artifact semantics.

Read this handbook when the question is about graph behavior, run identity,
replay fidelity, diff semantics, or DAG backend operations.

## Handbook Structure Contract

`docs/bijux-dag/` keeps one durable layout:

- exactly five section directories: `foundation`, `architecture`,
  `interfaces`, `operations`, `quality`
- each section contains exactly ten pages
- no additional nested chapter trees under `docs/bijux-dag/`

This keeps DAG documentation stable and easy to review across releases.

Topic migration coverage is documented in
[Documentation Standards](quality/documentation-standards.md).

## Main Paths

- [Foundation](foundation/index.md)
- [Architecture](architecture/index.md)
- [Interfaces](interfaces/index.md)
- [Operations](operations/index.md)
- [Quality](quality/index.md)
