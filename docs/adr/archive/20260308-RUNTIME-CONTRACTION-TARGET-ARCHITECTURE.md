# ADR: Runtime Contraction Target Architecture

## Status
Accepted

## Context
`bijux-dag-runtime` grew to include broad modeled surfaces that are not part of the deterministic execution kernel. This increased API surface and review cost.

## Decision
Runtime scope is constrained to execution-relevant surfaces:
- DAG execution planning and scheduling.
- Adapter invocation and capability checks used at execution time.
- Replay/runtime diagnostics that are executable and contract-backed.

All modeled or speculative platform surfaces must be quarantined behind explicit `experimental` namespaces and excluded from stable capability/help surfaces by default.

## Consequences
- Stable runtime exports remain focused and easier to verify.
- Speculative work remains possible without leaking into product truth surfaces.
- Governance gates can reject new broad runtime modules without ownership classification.
