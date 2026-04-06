---
title: Data Contracts
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Data Contracts

DAG data contracts cover graph definitions, execution plans, run traces,
artifact indices, and replay/diff classifications.

## Visual Summary

```mermaid
flowchart TD
    graph["graph contracts"] --> runtime["runtime execution contracts"]
    runtime --> run["run and node trace contracts"]
    run --> artifact["artifact index and lineage contracts"]
    artifact --> classify["replay and diff classification contracts"]
```

## Contract Families

- graph model and validation diagnostics
- execution plan and scheduler state representations
- node outcomes, run summaries, and timeline events
- artifact metadata, integrity proofs, and lineage links
- replay/diff classification payloads and reason codes

## Code Anchors

- `crates/bijux-dag-core/src/graph/model.rs`
- `crates/bijux-dag-core/src/contracts/error.rs`
- `crates/bijux-dag-runtime/src/runtime_core/`
- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-app/src/routes/response.rs`

## Contract Rules

- contract-bearing fields should stay explicit and test-covered
- identity-related field semantics require compatibility review
- classification states must remain machine-parseable

## Next Reads

- [Artifact Contracts](artifact-contracts.md)
- [Compatibility Commitments](compatibility-commitments.md)
- [Invariants](../quality/invariants.md)
