---
title: Data Contracts
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Data Contracts

This page explains the data shapes that let DAG definitions, runs, artifacts,
and comparisons stay inspectable across tools and time.

The important split is not model count. It is whether a contract describes the
graph itself, execution evidence, artifact evidence, or comparison outcomes.

## Contract Map

```mermaid
flowchart LR
    contracts["dag contracts"] --> graph["graph and validation models"]
    contracts --> plans["plan and execution models"]
    contracts --> state["run and trace models"]
    contracts --> artifacts["artifact and lineage models"]
    contracts --> compare["replay and diff outcomes"]
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

## Reading Rule

Use this page when a DAG change crosses graph, run, artifact, or diff
boundaries and the hard part is deciding which observable contract is at stake.

## Next Reads

- [Artifact Contracts](artifact-contracts.md)
- [Compatibility Commitments](compatibility-commitments.md)
- [Invariants](../quality/invariants.md)
