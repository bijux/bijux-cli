---
title: Execution Engine Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Execution Engine Contract

The execution engine owns orchestration. Backends own lifecycle-specific work
inside the backend contract boundary.

## Scope

This contract describes how `execute_with_backend` coordinates backend binding,
lifecycle sequencing, and attempt recording without blurring engine and backend
responsibilities.

## Engine responsibilities

The engine must:

- construct a `BackendBindingRequest` for each node
- create `BackendContext` with node id, attempt, arguments, environment, and
  declared outputs
- run lifecycle stages in the stable order `prepare -> launch -> observe -> finalize -> cleanup`
- reject undeclared backend outputs before treating an attempt as valid
- record one `ExecutionAttemptRecord` per completed node attempt

## Backend responsibilities

Backends must:

- expose a stable `name()` and `capabilities()`
- classify lifecycle failures through `BackendError`
- return `BackendLifecycleResult` with `status`, `exit_code`, `stdout`,
  `stderr`, and `produced_outputs`
- tolerate cleanup calls even when earlier lifecycle stages fail

## Cleanup and failure ordering

Cleanup is a contractual obligation, not an optional best effort.

- if lifecycle work fails, cleanup still runs
- if lifecycle succeeds but cleanup fails, the overall outcome is a cleanup
  failure
- if lifecycle fails, the primary lifecycle failure remains the returned error

## Proof surfaces

- `crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
- `crates/bijux-dag-runtime/tests/engine_flow_contract.rs`
- `docs/bijux-dag/architecture/engine-backend-responsibilities.md`

## Related tests

- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
- `crates/bijux-dag-runtime/tests/engine_flow_contract.rs`
- `crates/bijux-dev/tests/backend_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to engine/backend sequencing, cleanup ordering, or
attempt recording must update this document and the linked proof surfaces in the
same change.
