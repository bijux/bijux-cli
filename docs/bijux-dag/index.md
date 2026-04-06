---
title: DAG Handbook
audience: mixed
type: index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# DAG Handbook

`bijux-dag` is the graph execution and evidence subsystem in `bijux-core`. It
owns deterministic DAG semantics, run/artifact identity, replay classification,
and diff classification.

This handbook is optimized for operational questions that need hard answers:

- what changed (`graph`, `run`, or `artifact`)
- whether replay stayed equivalent, drifted, or is incomplete
- where ownership boundaries are between DAG crates

## Visual Summary

```mermaid
flowchart LR
    graph["graph definition"] --> run["run execution"]
    run --> evidence["run and artifact evidence"]
    evidence --> replay["replay classification"]
    replay --> diff["diff classification"]
    diff --> decision["release and operator decision"]
```

## Handbook Structure Contract

`docs/bijux-dag/` remains stable and reviewable:

- exactly five section directories: `foundation`, `architecture`,
  `interfaces`, `operations`, `quality`
- each section contains exactly ten pages
- no nested section trees under `docs/bijux-dag/`

## Code Anchors

- `crates/bijux-dag-cli/src/main.rs`
- `crates/bijux-dag-app/src/`
- `crates/bijux-dag-core/src/`
- `crates/bijux-dag-runtime/src/`
- `crates/bijux-dag-artifacts/src/`

## Main Paths

- [Foundation](foundation/index.md)
- [Architecture](architecture/index.md)
- [Interfaces](interfaces/index.md)
- [Operations](operations/index.md)
- [Quality](quality/index.md)

## Related Handbooks

- [Repository Handbook](../bijux-core/index.md)
- [CLI Handbook](../bijux-cli/index.md)
- [Maintainer Handbook](../bijux-dev/index.md)
