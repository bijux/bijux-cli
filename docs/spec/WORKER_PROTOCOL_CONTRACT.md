# Worker Protocol Contract

## Scope

Defines remote/distributed worker protocol semantics for lease, liveness, event delivery,
artifact handoff, and capability negotiation.

## Task lease semantics

- Lease identity is `(lease_id, run_id, node_id, worker_id)`.
- Lease expiration is absolute (`expires_unix_ms`) and must be monotonic.
- Recovery is allowed only inside explicit `recovery_grace_ms`.
- Reassignment outside recovery grace must create a new lease identity.

## Heartbeat semantics

- Heartbeats are classified as `healthy`, `delayed`, or `lost`.
- Classification is based on `interval_ms`, `delayed_threshold_ms`, and `timeout_ms`.
- Delayed heartbeat is an observable warning state and must not silently downgrade to healthy.

## Duplicate dispatch prevention

- Dispatch key is `(run_id, node_id)` for first-level dedupe.
- Duplicate dispatch acknowledgements are quarantined as duplicate observations and must not create new attempts.

## Worker identity model

Worker identity must include non-empty values for:

- `worker_id`
- `worker_version`
- `backend_kind`

## Artifact upload and commit semantics

- Upload and commit are distinct states.
- Artifact visibility requires a successful commit binding upload to run/node/attempt.
- Upload interruption or worker crash before commit must not appear as committed output.
- Transport checksum verification is mandatory for integrity-sensitive paths.

## Status event ordering guarantees

- Status events carry monotonic `sequence`.
- Out-of-order events are normalized before run-record projection.
- Duplicate sequence/status events are deduplicated and retained as duplicate observations.

## Cancellation delivery semantics

- Cancellation includes issuance timestamp and delivery deadline.
- Delivery beyond deadline is classified as missed-timing behavior.

## Worker version and capability negotiation

- Planner/worker version mismatch is rejectable and must be explicit.
- Worker pool capability negotiation validates resource and sandbox requirements before dispatch.

## Contract tests

- `crates/bijux-dag-runtime/tests/distributed_contracts.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
