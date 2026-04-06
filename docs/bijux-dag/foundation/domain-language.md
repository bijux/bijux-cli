---
title: Domain Language
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Domain Language

DAG decision quality depends on precise vocabulary. These terms are canonical
across docs, tests, and command output.

## Visual Summary

```mermaid
flowchart LR
    graph["graph identity"] --> run["run identity"]
    run --> artifact["artifact identity"]
    artifact --> replay["replay classification"]
    replay --> diff["diff classification"]
```

## Core Terms

- `graph identity`: canonical fingerprint of DAG semantics.
- `run identity`: one execution attempt under a graph/policy context.
- `artifact identity`: content-plus-policy identity for produced outputs.
- `equivalent`: required comparison scopes matched.
- `drift`: required comparison scope diverged.
- `incomplete`/`unknown`: required scope could not be resolved.

## Scope Terms

- `graph diff`: definition-level semantic comparison.
- `run diff`: execution behavior and outcome comparison.
- `artifact diff`: output payload identity comparison.
- `capability envelope`: backend/environment constraints used in classification.

## Code Anchors

- `crates/bijux-dag-core/src/analysis/fingerprint.rs`
- `crates/bijux-dag-runtime/src/replay/`
- `crates/bijux-dag-app/src/routes/diff_routes.rs`
- `crates/bijux-dag-artifacts/src/integrity/`

## Next Reads

- [Lifecycle Overview](lifecycle-overview.md)
- [Data Contracts](../interfaces/data-contracts.md)
- [Invariants](../quality/invariants.md)
