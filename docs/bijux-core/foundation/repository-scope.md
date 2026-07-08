---
title: Repository Scope
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Repository Scope

Use this page when you need to decide whether a question belongs in the
repository handbook at all.

The repository handbook is for the parts of `bijux-core` that cross package or
product boundaries. It is not where command semantics, DAG runtime behavior, or
maintainer implementation details should be explained a second time.

## What Belongs Here

| Topic | Why it belongs at the repository level |
| --- | --- |
| workspace membership and root build policy | it affects more than one crate family |
| shared documentation structure and publication | it governs how multiple handbooks fit together |
| cross-program contracts under `contracts/` | the same contract may constrain more than one product family |
| release, compatibility, and review rules | those rules must stay consistent across products |

## What Does Not Belong Here

- CLI runtime semantics that belong in `docs/bijux-cli/`
- DAG execution semantics that belong in `docs/bijux-dag/`
- maintainer implementation detail that belongs in `docs/bijux-dev/`

## Reader Shortcut

If the answer only needs one product handbook, stay out of the repository
handbook. Come here when the answer spans products, packages, contracts, or
release rules.

## What This Page Is Not Saying

- It is not claiming repository pages are more important than product pages.
- It is not saying root docs should duplicate crate-level behavior.
- It is not replacing ownership pages when you need the exact package family.

## Code Anchors

- `Cargo.toml`
- `Makefile`
- `contracts/`
- `mkdocs.yml`

## Continue Reading

- [Workspace Layout](workspace-layout.md)
- [Decision Rules](decision-rules.md)
- [Repository Handbook](../index.md)
