---
title: Repository Scope
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Repository Scope

`bijux-core` has repository-level documentation because some questions cannot
be answered honestly from a single product handbook. The root scope is not
"everything in the repo." It is the smaller set of questions that cross
products, packages, release boundaries, or shared contracts.

This page defines that line so the repository handbook stays useful instead of
becoming a vague duplicate of the CLI, DAG, or maintainer handbooks.

## Use The Repository Handbook When The Question Crosses Boundaries

| Question type | Why it belongs here |
| --- | --- |
| Which product family or crate owns this surface? | ownership routing crosses package and handbook boundaries |
| Which crates are public and which stay repository-internal? | publication intent is a root release concern |
| Which shared contracts constrain more than one product family? | contracts under `contracts/` often feed both docs and tests across the workspace |
| Which review, release, or compatibility rules apply to both `bijux` and `bijux-dag`? | those rules must stay consistent above product-level docs |
| How is the workspace laid out and why does that shape matter? | root structure affects contributors across the whole repository |

## Leave The Repository Handbook When One Owner Is Clear

Once the answer clearly belongs to one product or one maintainer surface, move
to the owning handbook:

- [CLI Handbook](../../bijux-cli/index.md) for `bijux` runtime behavior
- [DAG Handbook](../../bijux-dag/index.md) for graph, run, replay, and
  artifact behavior
- [Maintainer Handbook](../../bijux-dev/index.md) for repository automation,
  release proof, and governance tooling

## What The Repository Layer Actually Covers

The repository handbook is the shared layer between those handbooks. It exists
to explain:

- how the workspace is divided into public products and private support crates
- how contracts, docs, and release rules stay aligned across that divide
- which top-level directories and entrypoints are stable repository surfaces
- which root rules contributors must understand before changing more than one
  package family

## What Usually Falls Out Of Scope

These belong elsewhere unless the root boundary itself is the subject:

- command syntax and runtime behavior for `bijux`
- DAG authoring or execution semantics
- implementation detail for `bijux-dev` maintainer commands
- crate-local behavior that does not affect release, docs, contracts, or
  shared ownership

## A Practical Test

Stay here if at least one of these is true:

1. the answer needs more than one handbook
2. the change can affect more than one public product family
3. the reader needs the release or contract boundary, not just the behavior
4. the owning surface is a root directory, root Make target, or shared
   contract

If none of those are true, the repository layer is probably the wrong starting
point.

## Drift Signs

A repository page has drifted when it starts doing any of the following:

- teaching one command family in product-level detail
- restating DAG execution behavior that already belongs in the DAG handbook
- documenting maintainer internals that should live under `docs/bijux-dev/`
- listing code paths without explaining the shared repository decision they
  support

## Durable Anchors

The repository layer is grounded in a small set of root surfaces:

- `Cargo.toml` for workspace membership
- `Makefile` and `makes/` for root entrypoints
- `contracts/` for shared machine-readable truth
- `mkdocs.yml` for published handbook structure

## Continue Reading

- [Workspace Layout](workspace-layout.md)
- [Decision Rules](decision-rules.md)
- [Platform Overview](platform-overview.md)
- [Repository Handbook](../index.md)
