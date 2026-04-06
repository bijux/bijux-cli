---
title: Change Principles
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Change Principles

DAG change quality depends on explicit tradeoffs. These principles preserve
contract clarity while the implementation evolves.

## Visual Summary

```mermaid
flowchart LR
    proposal["change proposal"] --> classify["classify contract impact"]
    classify --> evidence["tests and docs evidence"]
    evidence --> review["scope and compatibility review"]
    review --> release["release decision with explicit risk"]
```

## Principles

- determinism before convenience shortcuts
- explicit contracts before implicit behavior
- inspectability before opaque optimization
- replay/diff truthfulness before cosmetic success signals
- honest capability bounds before universal parity claims

## High-Risk Change Areas

- identity and canonicalization behavior
- replay and diff outcome classification logic
- artifact integrity and lineage persistence
- scheduler fairness and execution policy defaults

## Code Anchors

- `crates/bijux-dag-core/src/analysis/`
- `crates/bijux-dag-runtime/src/replay/`
- `crates/bijux-dag-runtime/src/runtime_core/`
- `crates/bijux-dag-artifacts/src/integrity/`

## Next Reads

- [Change Validation](../quality/change-validation.md)
- [Review Checklist](../quality/review-checklist.md)
- [Release and Versioning](../operations/release-and-versioning.md)
