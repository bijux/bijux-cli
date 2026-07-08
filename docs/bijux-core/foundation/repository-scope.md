---
title: Repository Scope
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Repository Scope

`bijux-core` exists above any single crate, but that does not mean every
question belongs in the repository handbook. The repository layer is only the
right home when the answer depends on more than one product family, more than
one package boundary, or more than one published surface.

This page defines that boundary so the root docs stay useful instead of turning
into a duplicate of the CLI, DAG, or maintainer handbooks.

## What Belongs Here

| Topic | Why it belongs at the repository level |
| --- | --- |
| workspace membership and root build policy | it affects more than one crate family |
| shared documentation structure and publication | it governs how multiple handbooks fit together |
| cross-program contracts under `contracts/` | the same contract may constrain more than one product family |
| release, compatibility, and review rules | those rules must stay consistent across products |

## What Usually Does Not Belong Here

- CLI runtime semantics that belong in `docs/bijux-cli/`
- DAG execution semantics that belong in `docs/bijux-dag/`
- maintainer implementation detail that belongs in `docs/bijux-dev/`

## What The Repository Handbook Is For In Practice

Readers usually need the repository layer for one of four reasons:

- to understand how the workspace is split
- to understand how products and support crates relate to each other
- to understand which rules apply across both public product families
- to understand shared release, contract, and documentation boundaries

That is a narrower job than "explain the whole repository." It is the job of
explaining the shared surface between the handbooks.

## A Useful Shortcut

If the answer only needs one product handbook, stay out of the repository
handbook. Come here when the answer spans products, packages, contracts, or
release rules.

## Signs A Page Is Out Of Scope

A repository page has probably drifted out of scope when it:

- explains one command family in detail
- repeats DAG execution semantics already owned by `docs/bijux-dag/`
- documents maintainer implementation details better owned by `docs/bijux-dev/`
- teaches crate-local behavior instead of shared repository boundaries

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
- [Platform Overview](platform-overview.md)
- [Repository Handbook](../index.md)
