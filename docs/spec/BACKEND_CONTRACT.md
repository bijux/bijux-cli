---
title: Backend Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Backend Contract

`bijux-dag-runtime` binds node execution to explicit backend capabilities and a
stable execution lifecycle.

## Scope

This contract covers the backend execution surface in
`crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs`.

It does not promise that all modeled future backend kinds are production-ready.
Only the current local proof surfaces and their conformance tests are in scope
here.

## Backend kinds

Stable backend kinds are:

- `Shell`
- `Process`
- `Container`
- `RemoteFuture`

Runtime code may model future backend kinds, but only backends with explicit
conformance proof may participate in the current execution contract.

## Boundary truth

The current backend contract distinguishes capability and lifecycle ownership
from stronger sandbox claims.

- `Shell` and `Process` provide a local subprocess boundary, not a host sandbox
- `Container` can provide stronger engine-managed mount and no-network controls,
  but it is still not documented as a VM boundary
- `RemoteFuture` remains modeled in the runtime and must not be treated as a
  shipped public remote-worker security boundary without separate backend proof

## Capability binding

Backends are matched through `BackendBindingRequest` and `BackendCapabilities`.

- the engine must reject a backend whose `kind` does not satisfy the node’s
  required kind
- capability mismatch must surface as `BackendError::Capability`
- backend capability descriptors must remain visible through
  `backend_registry()`

## Lifecycle stages

The stable backend lifecycle is:

1. `prepare`
2. `launch`
3. `observe`
4. `finalize`
5. `cleanup`

The engine must not treat a backend run as successful unless the lifecycle
completes with valid outputs and cleanup succeeds.

## Error classification

Failure classes are part of the contract surface:

- `BackendError::Prepare`
- `BackendError::Launch`
- `BackendError::Observe`
- `BackendError::ObserveTimeout`
- `BackendError::Finalize`
- `BackendError::Cleanup`

These categories exist so operator evidence and tests can distinguish backend
failures without collapsing them into a generic runtime error.

## Output and environment rules

- `BackendContext` owns node id, attempt number, arguments, environment, and
  declared outputs
- declared output targets must be authorized before backend launch so malformed
  paths such as `../escape.txt` never become writable targets
- symlinked existing parent components in declared output paths must be
  rejected before backend launch
- undeclared outputs must fail backend finalization rather than being silently
  accepted
- environment shaping must remain explicit and must not leak ambient state into
  a backend that expects a controlled contract
- `clean-env` is an environment-shaping control, not a filesystem, network, or
  clock sandbox
- deny flags gate declared effects before execution starts; they do not imply
  syscall interposition after a backend has launched

## Conformance proof

The backend contract is backed by:

- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
- the parity proof
  `fake_and_process_like_backends_have_parity_on_basic_scenario`
- explicit failure tests for prepare, launch, observe timeout, finalize, and
  cleanup behavior

## Related tests

- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
- `crates/bijux-dev/tests/backend_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to backend kinds, lifecycle stages, error
classification, output validation, or registry semantics must update this
document and the linked conformance tests in the same change.
