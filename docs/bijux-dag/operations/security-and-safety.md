---
title: Security And Safety
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Security And Safety

Security and safety for DAG focus on controlled execution, artifact integrity,
and predictable failure handling.

## Visual Summary

```mermaid
flowchart LR
    inputs[input and config validation] --> execution[bounded execution]
    execution --> artifacts[artifact integrity checks]
    artifacts --> review[operator review and approval]
    review --> promotion[promotion decision]
```

## Safety Principles

- validate graphs and inputs before execution
- restrict runtime privileges to minimum required scope
- verify artifact integrity before downstream consumption
- favor fail-closed behavior for unknown mismatch categories

## Security Control Areas

- configuration and secret boundary discipline
- filesystem and storage write scope constraints
- tamper detection via hash and proof validation

## Code Anchors

- `crates/bijux-dag-app/src/routes/validate_routes.rs`
- `crates/bijux-dag-artifacts/src/integrity/proof.rs`
- `crates/bijux-dag-runtime/src/env/`

## Next Reads

- [Deployment Boundaries](deployment-boundaries.md)
- [Risk Register](../quality/risk-register.md)
- [Artifact Contracts](../interfaces/artifact-contracts.md)
