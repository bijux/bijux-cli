---
title: Invariants
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Invariants

Invariants protect the meaning of DAG execution and must not drift silently.

## Visual Summary

```mermaid
flowchart TD
    invariants[Core invariants]
    invariants --> identity[deterministic identity meaning]
    invariants --> transitions[state transitions are explicit]
    invariants --> lineage[artifacts remain attributable]
    invariants --> classification[failures are classifiable]
    invariants --> interfaces[interfaces remain structured]

    identity --> tests[test and contract coverage]
    transitions --> tests
    lineage --> tests
    classification --> tests
    interfaces --> tests
```

## Core Invariants

- canonical graph identity is stable for equivalent definitions
- run/replay identities remain attributable and non-ambiguous
- runtime fingerprints are derived from build identity, not from the shell
  location used to launch the binary
- diff classifications preserve mismatch semantics and group boundaries
- artifact indexes/proofs remain internally consistent and verifiable

## Invariant Breach Signals

- same graph yields different canonical fingerprint without rule change
- same binary yields different runtime fingerprint after only changing the
  working directory
- replay changes fidelity class without environment or input explanation
- diff reason-code meanings mutate without compatibility notice
- integrity validation accepts tampered or incomplete evidence

## Code Anchors

- `crates/bijux-dag-core/src/analysis/fingerprint.rs`
- `crates/bijux-dag-runtime/src/replay/`
- `crates/bijux-dag-artifacts/src/integrity/`

## Next Reads

- [Test Strategy](test-strategy.md)
- [Risk Register](risk-register.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
