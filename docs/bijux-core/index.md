---
title: Repository Handbook
audience: mixed
type: index
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Repository Handbook

This handbook owns repository-level conventions that span more than one crate
or program boundary.

Use this section when no single program handbook can answer the question on its
own.

## Pages In This Section

- [Platform Overview](platform-overview.md)
- [Repository Scope](repository-scope.md)
- [Workspace Layout](workspace-layout.md)
- [Package Map](package-map.md)
- [API and Schema Governance](api-and-schema-governance.md)
- [Local Development](local-development.md)
- [Testing and Validation](testing-and-validation.md)
- [Release and Versioning](release-and-versioning.md)
- [Documentation System](documentation-system.md)

## Use This Section For

- workspace boundaries and ownership rules
- shared schema and contract governance
- release and validation policy that applies across CLI and DAG

## Leave This Section For Program Handbooks When

- behavior is specific to one command surface
- behavior is specific to DAG runtime execution semantics
- a maintainer operation is local to tooling and not product behavior
