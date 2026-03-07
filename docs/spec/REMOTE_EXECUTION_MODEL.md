# Remote Execution Model

## Scope
This document defines the minimum remote execution model currently supported by
contracts and simulations in `bijux-dag`.

## Status boundary
- Implemented: local and process-like execution backends
- Simulated: remote identity/handoff model and backend contracts
- Not implemented: production Kubernetes/HPC job orchestration backend

Normative docs must not claim production Kubernetes/HPC execution support.

## Identity model
Every remote attempt identity must include:
- `run_id`
- `node_id`
- `attempt_id`
- `backend_id`

## Artifact handoff model
Remote artifact handoff includes:
- upload endpoint identity
- download endpoint identity
- integrity requirement flag
- stable mapping from attempt to artifact namespace

## Observability handoff model
Remote observability handoff includes:
- log stream mode
- trace event forwarding capability
- retention hint
- correlation IDs for run/node/attempt/backend

## Execution responsibility split
- Engine: attempt state machine, retry policy, failure classification
- Backend: submission, observation, cancellation, cleanup
- Storage: artifact and metadata persistence boundaries

## Compatibility notes
- Future remote backends must satisfy `ExecutionBackend` contract tests.
- Capability mismatches must fail at binding/planning boundaries.
- Worker lease/heartbeat/ordering semantics are governed by
  `docs/spec/WORKER_PROTOCOL_CONTRACT.md`.
- Delivery guarantees and hard vs best-effort boundaries are governed by
  `docs/spec/REMOTE_DELIVERY_GUARANTEES.md`.

## Verifying tests
- `crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
