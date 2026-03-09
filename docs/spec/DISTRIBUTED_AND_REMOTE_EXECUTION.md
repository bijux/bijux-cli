# DISTRIBUTED AND REMOTE EXECUTION

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/BATCH_EXECUTION_MODEL.md
# Batch Execution Model

## Scope
This document defines the batch execution shape for long-running remote jobs.
In this repository, batch/HPC execution is modeled and simulated, not production
executed.

## Boundary decision
Batch/HPC support is modeled as an execution backend family.
It is not modeled as an adapter payload and not a control-plane-only wrapper.

## Required batch job metadata
- `scheduler_id`
- `submission_time_unix_ms`
- `run_id`
- `node_id`
- `attempt_id`
- `resource_request`
- `status_mapping`

## Retry semantics
- Retry submits a new scheduler job with a new `attempt_id`.
- Attempt lineage links all retry submissions to a single node execution lineage.

## Cancellation semantics
- Runtime cancellation maps to scheduler cancellation request.
- Cancellation outcome is recorded as success/failure/unknown delivery.

## Output and log collection
- stdout/stderr collection is mapped into run attempt observability.
- declared output collection must complete before attempt is finalized success.
- delayed artifact availability is represented as pending collection state.

## Remote failure mapping
- stale status updates -> transient remote state error
- missing status updates -> unknown remote state error
- duplicate delivery -> idempotent state application required

## Long-run progress model
- batch attempts emit heartbeat records while active.
- heartbeat timeout policy determines stale-attempt detection.

## Recovery boundary
Controller restart recovery for active remote batch attempts is not implemented as
fully resumable execution in this repository. Restart detection must fail
explicitly and report unsupported recovery.

## Mode classification
- implemented: local, subprocess
- simulated: batch contract and fake batch backend
- aspirational: production Slurm/PBS/Kubernetes execution backend

## Verifying tests
- `crates/bijux-dag-runtime/tests/batch_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs`

## SOURCE: docs/spec/CONTAINER_EXECUTION_CONTRACT.md
# Container Execution Contract

## Scope
This contract defines the minimum container execution model for `bijux-dag`.
Container execution is modeled as a backend contract surface. It is not an
adapter payload format.

## Required fields
- `image`: immutable image reference or digest
- `command`: argv vector
- `env`: explicit environment map after policy shaping
- `mounts`: local-to-container path mappings
- `declared_outputs`: container-relative output paths
- `timeout_ms`: optional execution timeout

## Path model
- Local artifact roots are mounted into a declared container root.
- Output paths must be normalized and remain under declared output root.
- Traversal (`..`) and absolute host path escape are rejected.

## Output declaration model
- A container node is successful only when declared outputs are present
  relative to declared output roots.
- Missing declared outputs is a contract violation.

## Error mapping
- missing image -> backend preparation error
- launch failure -> backend launch error
- timeout -> execution timeout classification
- missing outputs -> artifact contract error

## Environment isolation model
- `clean_env` applies before mount/launch assembly.
- Allowlist and denylist patterns are applied to all final environment keys.
- Undeclared or denied keys are removed.

## Kubernetes scope
Kubernetes execution is not implemented as a runnable backend in this repo.
Kubernetes material is limited to contract/model definitions and simulation.

## Versioning and compatibility
- Additive field additions are backward-compatible.
- Required-field semantic changes require contract version bump and tests.

## Verifying tests
- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`

## SOURCE: docs/spec/DISTRIBUTED_COORDINATION_MODEL.md
# Distributed Coordination Model

## Scope boundary
Bijux DAG is currently a single-controller system. Distributed coordination is not an implemented production execution mode.

## Controller and backend responsibilities
- Controller responsibilities:
  - plan execution
  - scheduler state transitions
  - authoritative run metadata writes
  - terminal run decisions
- Backend and worker responsibilities:
  - execute attempts
  - emit observations (status, timestamps, logs, artifacts)
  - never finalize authoritative run status directly

## Single-writer rule
Run metadata has exactly one writer: the controller process for the run. Remote workers can emit observations only.

## Trace writers
Multiple writers are allowed for observational traces. Reconciliation ordering is controller-defined with sequence/time precedence and idempotent deduplication.

## Remote event semantics
- Out-of-order events: accepted, reconciled by sequence monotonicity.
- Duplicate events: idempotent.
- Missing completion: run remains non-terminal until timeout/cancellation/policy resolution.
- Inconsistent snapshots: terminal controller state is never reverted by later remote observations.

## Restart model
Controller restart recovery for remote coordination is simulation-only. Production-grade distributed recovery is not implemented.

## Reconciliation model
Controller reconciles remote observations into local run state using:
- per-node sequence high-water marks
- terminal-state immutability
- idempotent event application

## Not implemented boundary
- no distributed consensus for run state
- no multi-controller active coordination
- no authoritative remote state writers

## Dependency boundary rule
Any distributed-control work must preserve planner, scheduler, and storage contracts as the source of local semantics.

## SOURCE: docs/spec/DISTRIBUTED_EXECUTION_ARCHITECTURE_CONTRACT.md
# Distributed Execution Architecture Contract

## Purpose

Define expected behavior and verification surfaces for remote worker and distributed execution pathways.

## Required capability surfaces

- remote worker registration and identity
- worker capability reporting
- task dispatch and completion reporting
- failure and timeout reporting
- retry scheduling behavior
- artifact upload and download surfaces
- replay compatibility and provenance continuity

## Required robustness checks

- network failure behavior
- latency tolerance behavior
- stress and scalability behavior
- deterministic behavior under equivalent inputs
- telemetry and diagnostics coverage

## Governance artifacts

- distributed execution regression corpus
- distributed execution stress suite
- distributed execution benchmark report
- distributed execution telemetry report

## SOURCE: docs/spec/REMOTE_DELIVERY_GUARANTEES.md
# Remote Delivery Guarantees

## Scope

Defines delivery guarantees and reliability boundaries for remote/distributed worker protocol execution.

## Delivery guarantees

- Dispatch and status delivery are **at-least-once**.
- Exactly-once execution is not guaranteed by transport alone.
- Duplicate dispatch and duplicate acknowledgements must be handled by protocol dedup keys.

## Hard guarantees

- Lease expiration and recovery windows are explicit and machine-checkable.
- Heartbeat classification (`healthy`, `delayed`, `lost`) is deterministic.
- Status event ordering is normalized by `sequence` before run-record projection.
- Artifact commit visibility requires successful upload+commit binding.

## Best-effort guarantees

- Status reporting latency under network partition.
- Cancellation delivery latency in congested control links.
- Worker reconnect timing after crash or process restart.

## Operator implications

- Operators should treat delayed heartbeat as a degraded state, not failure completion.
- Replay and diff remain authoritative for final semantic outcomes.

## SOURCE: docs/spec/REMOTE_EXECUTION_MODEL.md
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

## SOURCE: docs/spec/WORKER_PROTOCOL_CONTRACT.md
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

## SOURCE: docs/spec/WORK_STEALING_SCHEDULING_BOUNDARIES.md
# Work-stealing scheduler boundaries

## Current boundaries

- Planning boundary: `ExecutionPlan` is produced before execution.
- Scheduling boundary: `Scheduler` selects ready nodes based on policy and runtime state.
- Executor boundary: `LocalExecutor` handles bounded submission and in-flight tracking.

## Future work-stealing design

- Introduce per-worker ready deques with global fallback queue.
- Preserve deterministic mode as the correctness baseline.
- Add a policy switch between deterministic and work-stealing runtime modes.
- Maintain stable event semantics independent of scheduler implementation.

## Contract requirements before implementation

- `SchedulerEventHook` coverage for eligible, blocked, and scheduled transitions.
- Queue isolation policy surface remains unchanged (`SingleQueue`, `GroupIsolated`).
- Execution checkpoints keep the same schema across scheduler implementations.

## SOURCE: docs/spec/appendices/runtime/DISTRIBUTED_COORDINATION_MODEL.md
# Distributed Coordination Model

## Scope boundary
Bijux DAG is currently a single-controller system. Distributed coordination is not an implemented production execution mode.

## Controller and backend responsibilities
- Controller responsibilities:
  - plan execution
  - scheduler state transitions
  - authoritative run metadata writes
  - terminal run decisions
- Backend and worker responsibilities:
  - execute attempts
  - emit observations (status, timestamps, logs, artifacts)
  - never finalize authoritative run status directly

## Single-writer rule
Run metadata has exactly one writer: the controller process for the run. Remote workers can emit observations only.

## Trace writers
Multiple writers are allowed for observational traces. Reconciliation ordering is controller-defined with sequence/time precedence and idempotent deduplication.

## Remote event semantics
- Out-of-order events: accepted, reconciled by sequence monotonicity.
- Duplicate events: idempotent.
- Missing completion: run remains non-terminal until timeout/cancellation/policy resolution.
- Inconsistent snapshots: terminal controller state is never reverted by later remote observations.

## Restart model
Controller restart recovery for remote coordination is simulation-only. Production-grade distributed recovery is not implemented.

## Reconciliation model
Controller reconciles remote observations into local run state using:
- per-node sequence high-water marks
- terminal-state immutability
- idempotent event application

## Not implemented boundary
- no distributed consensus for run state
- no multi-controller active coordination
- no authoritative remote state writers

## Dependency boundary rule
Any distributed-control work must preserve planner, scheduler, and storage contracts as the source of local semantics.

## SOURCE: docs/spec/appendices/runtime/DISTRIBUTED_EXECUTION_ARCHITECTURE_CONTRACT.md
# Distributed Execution Architecture Contract

## Purpose

Define expected behavior and verification surfaces for remote worker and distributed execution pathways.

## Required capability surfaces

- remote worker registration and identity
- worker capability reporting
- task dispatch and completion reporting
- failure and timeout reporting
- retry scheduling behavior
- artifact upload and download surfaces
- replay compatibility and provenance continuity

## Required robustness checks

- network failure behavior
- latency tolerance behavior
- stress and scalability behavior
- deterministic behavior under equivalent inputs
- telemetry and diagnostics coverage

## Governance artifacts

- distributed execution regression corpus
- distributed execution stress suite
- distributed execution benchmark report
- distributed execution telemetry report
