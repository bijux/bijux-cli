# Execution Engine Contract

## Scope

Defines separation between orchestration engine and backend effect drivers.

## Responsibilities

Engine responsibilities:
- plan-driven orchestration of node attempts and state transitions
- scheduling integration and retry policy application
- attempt record generation
- run-level finalization and invariant checks

Backend responsibilities:
- process/container launch details
- stdout/stderr capture implementation
- backend-specific environment shaping
- backend-specific cleanup operations

Engine must not own backend-specific command construction.

## Backend Contract

Backends implement lifecycle hooks:
- `prepare`
- `launch`
- `observe`
- `finalize`
- `cleanup`

Backends must declare typed capabilities and kind.

## Capability Binding

Unsupported backend requirements must fail during planning/binding stage with
capability errors. They must not fail as late opaque runtime crashes.

## Attempt Record Contract

Attempt-level record is separate from node-level result and includes:
- `node_id`
- `attempt`
- `backend_kind`
- `status`
- `exit_code`

## Backend Types

Current backend classes in contract:
- `shell`
- `process`
- `container`
- `remote_future`

## Test Requirements

A backend implementation must satisfy:
- fake backend deterministic contract tests
- parity tests against process-like backend for agreed scenarios
- prepare/finalize/cleanup failure-path tests

## Governance Rule

New backend kinds cannot land without:
- backend contract tests
- fake-backend parity evidence
- explicit capability declaration

## Verifying Surfaces

- `crates/bijux-dag-runtime/src/execution_backend.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
- `bijux-dev-dag repo` suite `backend-contract`
