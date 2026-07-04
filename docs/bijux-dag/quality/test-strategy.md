---
title: Test Strategy
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Test Strategy

DAG test strategy prioritizes correctness of graph processing, runtime
execution, artifact integrity, and replay/diff semantics.

## Visual Summary

```mermaid
flowchart TB
  E2E[End-to-end and workflow tests]
  INT[Integration and contract tests]
  UNIT[Unit and focused logic tests]

  E2E --> INT
  INT --> UNIT

  UNIT --> Invariants[Fast invariant coverage]
  INT --> Boundaries[Boundary validation]
  E2E --> UserPaths[Critical operator paths]
```

## Test Layers

- unit tests for core parsing, lowering, and identity behavior
- integration tests for app command routes and run flows
- contract tests for replay/diff schema lockstep and semantics
- regression snapshots for human-readable explain surfaces

## Required Coverage Areas

- canonical graph parsing and validation failure modes
- execution plan and runtime fidelity classification
- artifact index/proof generation and verification paths
- runtime build identity and fingerprint stability across working directories
- replay and diff mismatch grouping and reason-code stability

## Code Anchors

- `crates/bijux-dag-core/tests/`
- `crates/bijux-dag-runtime/tests/`
- `crates/bijux-dag-app/tests/replay_diff_hardening_contract.rs`

## Next Reads

- [Change Validation](change-validation.md)
- [Invariants](invariants.md)
- [Known Limitations](known-limitations.md)
