# Control plane model

## Current boundary

`bijux-dev-dag` is the repository control plane. It governs local developer workflows, contract checks, and release verification.

## Future product boundary

`dag-api` is the planned service control plane. It will own persistent DAG registry, schedules, run submission, and policy evaluation for shared environments.

Typed API resource families include DAG, DAG version, run, node attempt, artifact, schedule, queue, policy, and audit event.

## Responsibilities split

- Repository control plane (`bijux-dev-dag`):
  - developer checks
  - contract execution
  - repository policy and release verification
- Runtime control plane (inside runtime contracts):
  - typed run-control operations
  - schedule compile and submission contracts
  - run/audit artifact generation
- Service control plane (`dag-api` roadmap):
  - multi-user registry and authorization
  - durable scheduler and queue management
  - remote run-control APIs

## Publication workflow contract

DAG versions move through:

- `draft`
- `validated`
- `active`
- `deprecated`
- `retired`

## Version selection policies

- `run latest`
- `run pinned`
- `run compatible`

## Validation service boundary

Validation is a typed service contract used consistently by CLI, scheduler, and future API surfaces.

## Audit semantics for mutating operations

Mutating control-plane actions (`submit`, `cancel`, `pause`, `resume`, `retry`, `replay`, `export`) must emit an audit record containing:

- action name
- status
- effect classification
- timestamp

Repository control-plane execution currently persists this audit stream under `artifacts/reports/control-plane-audit.jsonl`.

## Governance rule

Every public control-plane operation must map to typed request and typed response contracts.

Control-plane list/query APIs use typed pagination and filter contracts, and mutable resources carry optimistic concurrency markers.

## Environment model

Typed environment modes:

- local
- ci
- staging
- production
- airgapped

Environment values support inheritance and explicit overrides.

## Minimal control-plane MVP

MVP includes:

- DAG publish/inspect lifecycle
- run submit/cancel/pause/resume/retry/replay/verify
- schedule create/update/suspend/preview/audit
- artifact inspect/export/verify/lineage/retention action
- audit event persistence

MVP excludes:

- distributed execution orchestration
- HA/sharded scheduler topology management
