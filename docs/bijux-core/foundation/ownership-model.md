---
title: Ownership Model
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Ownership Model

Ownership in `bijux-core` is explicit on purpose. The repository is healthiest
when every behavior claim names one owner and every root rule explains why it
is above package scope.

```mermaid
flowchart LR
    subgraph OwnedByRepository
        root_docs[root handbook boundaries]
        shared_rules[cross-program rules]
    end

    subgraph OwnedByProducts
        cli_docs[CLI product behavior]
        dag_docs[DAG product behavior]
    end

    subgraph OwnedByMaintainer
        dev_docs[maintainer automation behavior]
    end

    root_docs --> shared_rules
    cli_docs -. must not redefine .-> shared_rules
    dag_docs -. must not redefine .-> shared_rules
    dev_docs -. must not redefine .-> cli_docs
```

## Ownership Rules

- product behavior belongs to CLI or DAG package handbooks
- repository-health automation belongs to `bijux-dev`
- root docs describe cross-program rules and boundaries, not package internals
- cross-package changes must update every affected handbook branch

## Boundary Violations

- root pages describing package-local behavior in detail
- maintainer docs redefining end-user command semantics
- package docs making repository-wide policy claims without root anchors

## Next Reads

- [Package Map](package-map.md)
- [Decision Rules](decision-rules.md)
- [Package Ownership](../governance/package-ownership.md)
