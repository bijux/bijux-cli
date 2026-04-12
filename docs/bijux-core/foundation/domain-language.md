---
title: Domain Language
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Domain Language

The repository uses a small set of terms repeatedly. They should stay stable so
readers are not forced to reinterpret the docs tree every time they move
between handbooks.

## Durable Terms

- `repository handbook`: root docs for cross-program rules and ownership
- `product handbook`: CLI or DAG docs for owned runtime behavior
- `maintainer handbook`: docs for repository-health automation and release work
- `package`: the concrete code ownership boundary under `crates/`
- `contract`: machine-checkable rule or schema that other surfaces rely on
- `evidence`: outputs, reports, or checks that support a review or release

## Naming Rule

Prefer names that still explain intent when read out of context two years
later. Avoid labels that only make sense relative to a temporary migration or
iteration.

## Next Reads

- [Documentation System](documentation-system.md)
- [Change Principles](change-principles.md)
- [Repository Scope](repository-scope.md)
