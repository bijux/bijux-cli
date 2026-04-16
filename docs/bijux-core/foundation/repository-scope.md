---
title: Repository Scope
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Repository Scope

The repository root owns the things that cross package boundaries. It does not
own command semantics that already belong to the CLI handbook or execution
semantics that already belong to the DAG handbook.

```mermaid
flowchart TD
    A[Proposed repository-level change] --> B{Spans package boundaries?}
    B -->|No| C[Keep change in owning package handbook]
    B -->|Yes| D{Is it root policy, docs shape, contracts, or release rules?}
    D -->|Yes| E[Repository scope]
    D -->|No| C
    E --> E1[workspace and root build policy]
    E --> E2[shared docs and publication]
    E --> E3[cross-program contracts]
    E --> E4[release and compatibility rules]
```

## In Scope

- workspace membership and root build policy
- shared documentation structure and publication
- cross-program contracts under `contracts/`
- release, compatibility, and review rules that span more than one handbook

## Out Of Scope

- CLI runtime semantics that belong in `docs/bijux-cli/`
- DAG execution semantics that belong in `docs/bijux-dag/`
- maintainer implementation detail that belongs in `docs/bijux-dev/`

## Code Anchors

- `Cargo.toml`
- `Makefile`
- `contracts/`
- `mkdocs.yml`

## Next Reads

- [Workspace Layout](workspace-layout.md)
- [Decision Rules](decision-rules.md)
- [Repository Handbook](../index.md)
