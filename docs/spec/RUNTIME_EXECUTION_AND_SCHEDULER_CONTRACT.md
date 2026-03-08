# Runtime execution and scheduler contract

**What this spec is not**: benchmark policy, release policy, or architectural rationale.

## Scope

Canonical contract for runtime execution semantics:

- command execution and state transitions
- scheduler behavior and transition model
- cache semantics at runtime
- distributed coordination expectations
- concurrency and safety boundaries
- fault tolerance surface

## Consolidated runtime rules

- Runtime execution is deterministic for equivalent graph/planning inputs.
- Node lifecycle transitions are explicit and legal-state bounded.
- Scheduler tie-break and ready-set behavior are stable within stable surface declarations.
- Cache use/reuse is governed by declared cache identity inputs and proof metadata.
- Backend/runtime boundaries are enforced through stable interfaces and explicit capability checks.
- Distributed worker observations are reconciled by controller and must not violate controller ownership.

## Evidence and implementation links

- Runtime and scheduler implementation: `crates/bijux-dag-runtime`, `crates/bijux-dag-app`
- Conformance suites: scheduler/scheduling contracts, fault tolerance checks, conformance test suites
- Traces and manifests under `crates/bijux-dag-app/tests` and `evidence/battle`

## Canonical appendices

- [runtime semantics](./appendices/runtime/RUNTIME_SEMANTICS_CONTRACT.md)
- [execution semantics](./appendices/runtime/EXECUTION_SEMANTICS_CONTRACT.md)
- [execution engine](./appendices/runtime/EXECUTION_ENGINE_CONTRACT.md)
- [execution acceptance gates](./appendices/runtime/EXECUTION_ACCEPTANCE_GATES.md)
- [fault tolerance](./appendices/runtime/RUNTIME_FAULT_TOLERANCE_CONTRACT.md)
- [scheduler contract and transitions](./appendices/runtime/SCHEDULER_CONTRACT.md)
- [cache contracts](./appendices/runtime/CACHE_CONTRACT.md)
