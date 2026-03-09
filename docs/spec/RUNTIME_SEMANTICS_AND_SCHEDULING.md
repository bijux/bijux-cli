# RUNTIME SEMANTICS AND SCHEDULING

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/EXECUTION_ACCEPTANCE_GATES.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/EXECUTION_ACCEPTANCE_GATES.md](./appendices/runtime/EXECUTION_ACCEPTANCE_GATES.md)

## SOURCE: docs/spec/EXECUTION_ENGINE_CONTRACT.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/EXECUTION_ENGINE_CONTRACT.md](./appendices/runtime/EXECUTION_ENGINE_CONTRACT.md)

## SOURCE: docs/spec/EXECUTION_SEMANTICS_CONTRACT.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/EXECUTION_SEMANTICS_CONTRACT.md](./appendices/runtime/EXECUTION_SEMANTICS_CONTRACT.md)

## SOURCE: docs/spec/REFERENCE_RUNTIME.md
# Reference Runtime

The reference runtime is the authoritative implementation for:

- planner execution lowering
- scheduler dispatch semantics
- node execution and state transitions
- artifact commit and manifest finalization

Adapters are extension surfaces. They must not redefine core execution meaning.

## SOURCE: docs/spec/RUNTIME_ALLOWED_DEPENDENCIES.md
# Runtime Allowed Dependencies

This document defines runtime crate dependency boundaries.

## Runtime crate (`bijux-dag-runtime`) allowed direct dependencies

- workspace crates: `bijux-dag-core`, `bijux-dag-artifacts`
- runtime support crates: `serde`, `serde_json`, `sha2`, `hex`, `thiserror`, `ctrlc`

## Runtime disallowed dependency classes

- CLI and command-routing crates (`clap`, `bijux-dag-cli`, `bijux-dag-app`)
- network/server orchestration crates (`axum`, `warp`, `octocrab`, `reqwest`, `git2`)

## Enforcement

- `crates/bijux-dev-dag/tests/no_cli_in_runtime.rs`
- `crates/bijux-dev-dag/tests/runtime_contraction_contracts.rs`
- `crates/bijux-dev-dag/tests/dependency_boundary_contracts.rs`

## SOURCE: docs/spec/RUNTIME_ARCHITECTURE_CLEANUP_CONTRACT.md
# Runtime Architecture Cleanup Contract

## Purpose

Define required cleanup and governance expectations for runtime architecture quality.

## Required architecture controls

- explicit runtime module responsibilities and boundaries
- ownership classification for runtime modules
- dependency graph hygiene and boundary enforcement
- duplicate helper detection and reduction
- oversized module tracking and split rationale enforcement
- low-value or unused runtime paths removal tracking

## Required verification surfaces

- module boundary architecture tests
- module dependency regression tests
- runtime architecture invariants tests
- runtime architecture regression fixtures

## Required observability artifacts

- runtime module coverage report
- runtime module complexity report
- runtime architecture telemetry report
- runtime architecture health dashboard

## SOURCE: docs/spec/RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md
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

## SOURCE: docs/spec/RUNTIME_FAULT_TOLERANCE_CONTRACT.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/RUNTIME_FAULT_TOLERANCE_CONTRACT.md](./appendices/runtime/RUNTIME_FAULT_TOLERANCE_CONTRACT.md)

## SOURCE: docs/spec/RUNTIME_PUBLIC_API_BOUNDARY.md
# Runtime Public API Boundary

## Sacred public runtime surfaces
- execution engine orchestration entrypoints
- scheduler contract surfaces
- typed run and node result models
- policy and selector inputs
- trace and invariant outputs

## Public API restrictions
- speculative modules must not be re-exported as first-class public runtime APIs.
- roadmap or product strategy models remain internal support types.
- control-plane semantics remain outside runtime core ownership.

## Facade modules
- `runtime`: core runtime orchestration facade
- `adapters`: adapter-related runtime facade
- `execution`: execution-path facade (plan, backend, executor)

## SOURCE: docs/spec/RUNTIME_SEMANTICS_CONTRACT.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/RUNTIME_SEMANTICS_CONTRACT.md](./appendices/runtime/RUNTIME_SEMANTICS_CONTRACT.md)

## SOURCE: docs/spec/SCHEDULER_CONTRACT.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/SCHEDULER_CONTRACT.md](./appendices/runtime/SCHEDULER_CONTRACT.md)

## SOURCE: docs/spec/SCHEDULER_STATESPACE_CONTRACT.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/SCHEDULER_STATESPACE_CONTRACT.md](./appendices/runtime/SCHEDULER_STATESPACE_CONTRACT.md)

## SOURCE: docs/spec/SCHEDULER_STATE_TRANSITIONS.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/SCHEDULER_STATE_TRANSITIONS.md](./appendices/runtime/SCHEDULER_STATE_TRANSITIONS.md)

## SOURCE: docs/spec/SCHEDULER_WORKLOAD_MANAGEMENT.md
# Scheduler workload management contracts

This document defines enterprise scheduling contracts for calendars, fairness, admission control, SLA policy, and workload simulation.

## Calendar and suppression controls

- `DagCalendar` defines timezone-aware blackout and holiday behavior.
- `BlackoutWindow` and `HolidayPolicy` encode operational suppression windows.
- `EnvironmentSuppression` allows environment-specific schedule suppression.

## Backfill orchestration and throttling

- `PartitionBackfillOrchestration` models partition-aware backfill planning.
- `BackfillThrottlingPolicy` reserves live capacity while limiting backfill submissions.
- `compute_partition_backfill_batches` and `apply_backfill_throttling` provide deterministic planning primitives.

## Fairness, service classes, and admission control

- `FairnessAlgorithm` and `StarvationPreventionPolicy` define anti-starvation controls.
- `ServiceClass` provides workload intent (`interactive`, `batch`, `archival`, `critical`).
- `QueueAdmissionPolicy` provides queue admission gates under resource pressure.

## Priority and batching behavior

- `WeightedPriorityPolicy` defines weighted priority scheduling with deterministic tie-breaks.
- `weighted_priority_tie_break_order` orders submissions by weight, then stable deterministic fields.
- `RunBatchPolicy` and `run_batches` define optional grouped run dispatch.
- `ConcurrencyScope` defines governance scope boundaries for limits.

## Trigger buffering, previews, and conflict detection

- `DependencyTriggerBufferPolicy` defines dedup/buffering controls for bursty upstream triggers.
- `materialize_next_runs` provides deterministic next-`N` schedule previews.
- `detect_cron_conflicts` detects concentrated cron windows.
- `deduplicate_trigger_events` defines duplicate trigger key handling.

## Overrides, suppression annotations, and SLA

- `ScheduleSuppressionAnnotation` and `ScheduleOverrideRecord` preserve operator/audit context.
- `SlaPolicy` defines expected start/finish and latency budgets.
- `SchedulerSlaMetrics` + `SchedulerAlertRule` support SLA miss and saturation alerting.

## Simulation and maturity tracking

- `SchedulingSimulationSuite` defines simulation intent and fixture sets.
- `CrossSchedulerCompatibility` tracks compatibility assumptions for HA/sharded futures.
- `SchedulerMaturityMatrix` tracks readiness: local-only, durable, multi-queue, backfill, HA.

Fixtures:

## SOURCE: docs/spec/appendices/runtime/EXECUTION_ACCEPTANCE_GATES.md
# Execution acceptance gates

Required acceptance checks:
- Node and run state-machine legality checks.
- Deterministic scheduling under `jobs=1` and `jobs>1`.
- Stable ready-node tie-break ordering.
- Deterministic failure propagation behavior.
- Retry backoff metadata persisted in node traces.
- Timeout failures distinguishable from execution failures.
- Cancellation writes complete final manifest.
- Selection/exclusion runs emit manifest and trace completeness.
- Replay behavior does not depend on ambient state.
- Latest symlink updates preserve historical run integrity.
- Run ID collision handling is deterministic.
- `clean_env` and `deny_env` interactions are deterministic.
- `deny_network` behavior is consistent across adapter classes.
- Output validation covers missing, extra, duplicate, malformed outputs.
- Manifest node totals must equal per-node trace totals.

## SOURCE: docs/spec/appendices/runtime/EXECUTION_ENGINE_CONTRACT.md
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

- `docs/spec/BACKEND_CONTRACT.md`
- `crates/bijux-dag-runtime/src/execution_backend.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
- `bijux-dev-dag repo` suite `backend-contract`

## SOURCE: docs/spec/appendices/runtime/EXECUTION_SEMANTICS_CONTRACT.md
# Execution semantics contract

## Scope
Defines execution semantics for planning, ordering, failure/retry behavior, selection, replay, and run integrity invariants.

## Planning and ordering
- Plan construction and execution ordering are independent contract surfaces.
- Deterministic workloads must produce equivalent outcomes for `jobs=1` and `jobs>1`.
- Equal-priority ready nodes use stable tie breaking.

## Failure and retry semantics
- Failure propagation is deterministic from state and dependency graph.
- Retry attempts and backoff metadata are persisted in trace artifacts.
- Timeout failures are explicitly distinguishable from execution failures.
- Cancellation must emit a complete final manifest.

## Selection and replay semantics
- Selection/exclusion still emits trace-complete manifests.
- Replay must not consult ambient state outside recorded artifacts and policy/config inputs.

## Integrity semantics
- Latest symlink updates must not mutate historical run directories.
- Run ID collision handling must be deterministic and safe.
- Manifest node totals must equal trace status totals.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`
- `evidence/battle/workflows/happy_path/*`

## Versioning and change policy
Execution semantic changes require matching updates to state-machine docs, runtime tests, and replay behavior tests in one change.

## SOURCE: docs/spec/appendices/runtime/RUNTIME_ALLOWED_DEPENDENCIES.md
# Runtime Allowed Dependencies

This document defines runtime crate dependency boundaries.

## Runtime crate (`bijux-dag-runtime`) allowed direct dependencies

- workspace crates: `bijux-dag-core`, `bijux-dag-artifacts`
- runtime support crates: `serde`, `serde_json`, `sha2`, `hex`, `thiserror`, `ctrlc`

## Runtime disallowed dependency classes

- CLI and command-routing crates (`clap`, `bijux-dag-cli`, `bijux-dag-app`)
- network/server orchestration crates (`axum`, `warp`, `octocrab`, `reqwest`, `git2`)

## Enforcement

- `crates/bijux-dev-dag/tests/no_cli_in_runtime.rs`
- `crates/bijux-dev-dag/tests/runtime_contraction_contracts.rs`
- `crates/bijux-dev-dag/tests/dependency_boundary_contracts.rs`

## SOURCE: docs/spec/appendices/runtime/RUNTIME_FAULT_TOLERANCE_CONTRACT.md
# Runtime Fault Tolerance Contract

## Purpose

Define required runtime fault tolerance guarantees for crash recovery, restart continuation, state persistence, failure detection, and resilience telemetry.

## Required fault tolerance coverage

- runtime crash recovery and restart continuation
- runtime state persistence and scheduler restart behavior
- worker reconnect and artifact recovery behavior
- replay, cancellation, and event-log recovery behavior
- partial-run recovery and explicit failure detection behavior
- resilience and recovery-latency benchmark coverage
- failure-injection and crash-simulation verification
- restart determinism and resilience telemetry coverage

## Required governance artifacts

- runtime fault tolerance regression corpus
- runtime fault tolerance verification suite
- runtime resilience benchmark report
- runtime recovery latency report
- runtime fault tolerance telemetry report
- runtime fault tolerance coverage report

## SOURCE: docs/spec/appendices/runtime/RUNTIME_PUBLIC_API_BOUNDARY.md
# Runtime Public API Boundary

## Sacred public runtime surfaces
- execution engine orchestration entrypoints
- scheduler contract surfaces
- typed run and node result models
- policy and selector inputs
- trace and invariant outputs

## Public API restrictions
- speculative modules must not be re-exported as first-class public runtime APIs.
- roadmap or product strategy models remain internal support types.
- control-plane semantics remain outside runtime core ownership.

## Facade modules
- `runtime`: core runtime orchestration facade
- `adapters`: adapter-related runtime facade
- `execution`: execution-path facade (plan, backend, executor)

## SOURCE: docs/spec/appendices/runtime/RUNTIME_SEMANTICS_CONTRACT.md
# Runtime semantics contract

## Scope

This contract defines core runtime semantics for scheduling, retries, failures, cache behavior, replay, and audit traces.

## Deterministic scheduling

- Ready nodes must be ordered deterministically.
- Tie-break order must be stable for identical priority.
- Fairness must prevent indefinite starvation.

## Node execution semantics

- Retry eligibility is bounded by explicit max attempts.
- Timeout is terminal for the current attempt.
- Cancellation is terminal once node terminal state is reached.
- Dependency resolution requires all required upstream nodes.

## Artifact and cache semantics

- Artifact commit requires complete write and manifest synchronization.
- Cache reuse requires fingerprint, schema, and proof metadata.
- Cache invalidation is required on policy, adapter version, or output schema change.

## Replay and manifest semantics

- Replay equivalence is validated by semantic fingerprint parity.
- Run manifest validity requires run header, trace index, outputs index, and consistent totals.

## Recovery, lineage, and failure semantics

- Recovery is required when checkpoints exist without terminal completion.
- Artifact lineage must exist for all referenced outputs.
- Failure classification must be explicit and machine-readable.

## Runtime audit and trace semantics

- Runtime emits append-only audit events.
- Trace event categories are countable by stable category key.

## SOURCE: docs/spec/appendices/runtime/SCHEDULER_CONTRACT.md
# Scheduler Contract

## Scope

This document defines scheduler semantics for bijux-dag runtime execution.
The scheduler contract covers ready-queue behavior, tie-breaking, retries,
cache/skip/failure downstream readiness semantics, and scheduler debug artifacts.

## Canonical Unit

- Canonical scheduling unit: `node`
- Attempt handling is modeled as node lifecycle events with retry requeue.
- Unit identity remains node-stable across retries; attempt updates are modeled in event detail.

## Runtime Model

- Scheduler model: `event-driven`
- Ready queue source: dependency indegree transitions to zero
- Tie-breaking for equally ready nodes: lexical order on `node_id`
- Priority model: no dynamic priority class in runtime node scheduler

## Readiness Semantics

- A downstream node becomes ready when all required predecessors are satisfied.
- Satisfaction events:
- `success`
- `cached`
- `skipped`
- Failure propagation modes:
- `fail_fast`: failed predecessor does not satisfy downstream readiness.
- `isolate_branch`: failed predecessor is treated as branch-local and may unlock
  independent downstream behavior.
- `continue_independent`: equivalent downstream readiness behavior to
  `isolate_branch` for current runtime.
- `quorum_like_future`: non-fail-fast behavior reserved for future quorum policy.

## Retry Semantics

- Retry enters `retry_queue` and must be requeued into `ready_queue` explicitly.
- Retry requeue must preserve deterministic tie-breaking.
- A node cannot exist in both `ready_queue` and `retry_queue` simultaneously.

## Scheduler State Model

Runtime exposes a dedicated state type:

- `SchedulerState`
- Owns indegree counters, adjacency, ready queue, retry queue, completion map,
  and scheduler event log.
- Provides explicit completion transitions:
- `complete_success`
- `complete_cached`
- `complete_skipped`
- `complete_failed`

## Event Log Model

Scheduler debug events are structured as:

- `sequence`
- `kind`
- `node_id`
- optional `detail`

Event kinds:

- `node_ready`
- `node_scheduled`
- `node_blocked_by_budget`
- `node_retry_queued`
- `node_retry_requeued`
- `node_cached`
- `node_skipped`
- `node_failed`

Debug mode requirement:

- Scheduler state exposes structured event log via `SchedulerState::events()`.
- Event sequencing is strictly increasing and timeline-reconstructable.

## Determinism Requirements

- For a fixed graph and fixed event sequence, `ready_queue` evolution is deterministic.
- A downstream node becomes ready at most once.
- Concurrency level (`jobs`, `max_parallelism`) does not change scheduler semantic node set.
- Cancellation prevents new scheduling batches.
- Timeout is represented as a scheduling timeout outcome and not conflated with node failure.

## Invariants

Scheduler invariants required in code and tests:

- Event sequence numbers are unique and monotonically increasing.
- No node can be in both retry queue and ready queue.
- Retry requeue does not duplicate nodes.

## Timeline Artifact

Runtime emits scheduler timeline data in run artifacts. Control plane command:

- `bijux-dev-dag dag scheduler-timeline --run-dir <path>`

This command emits scheduler-relevant timeline entries from
`observability.timeline.json` for completed runs.

## Legal state transitions

Legal scheduler state transitions are documented in:

- `docs/spec/SCHEDULER_STATE_TRANSITIONS.md`

## Versioning

- Contract version policy: additive-only for required fields within `v1`.
- Breaking semantic changes require a new major contract version.

## Verifying Tests and Checks

- `crates/bijux-dag-runtime/tests/scheduler_contract.rs`
- `bijux-dev-dag repo run --domain governance` (`scheduler-invariants` suite)

## SOURCE: docs/spec/appendices/runtime/SCHEDULER_STATESPACE_CONTRACT.md
# Scheduler state-space contract

## Scope
Defines legal node/run transitions and scheduler-policy determinism constraints.

State-space constraints:
- Node transitions must follow formal node state machine legality.
- Run transitions must follow formal run state machine legality.
- Illegal transitions are contract-test failures.

Determinism constraints:
- Dispatch outcome for deterministic workloads is independent of thread count.
- Failure propagation and retry sequencing are replay-explainable.

Policy constraints:
- `clean_env` and `deny_env` interactions are deterministic.
- `deny_network` behavior must be enforced consistently across shell and container adapters.

## Related tests
- `crates/bijux-dag-runtime/tests/state_machine_contract.rs`
- `crates/bijux-dag-runtime/tests/scheduler_determinism.rs`

## Versioning and change policy
State transition or policy interaction changes require explicit update of legal-transition tests and compatibility notes.

## SOURCE: docs/spec/appendices/runtime/SCHEDULER_STATE_TRANSITIONS.md
# Scheduler state transitions

## Scheduler work item lifecycle

- `ready` -> `scheduled`
- `scheduled` -> `completed_success`
- `scheduled` -> `completed_cached`
- `scheduled` -> `completed_skipped`
- `scheduled` -> `completed_failed`
- `scheduled` -> `retry_queued`
- `retry_queued` -> `retry_requeued`
- `retry_requeued` -> `scheduled`

## Transition constraints

- a downstream node may transition to `ready` at most once per run attempt lineage
- no node may exist in both retry queue and ready queue simultaneously
- cancellation blocks new scheduling decisions
- scheduler timeout returns timeout scheduling outcome without mutating failure classification

## Terminal scheduling outcomes

- `completed_success`
- `completed_cached`
- `completed_skipped`
- `completed_failed`

## SOURCE: docs/spec/appendices/runtime/SCHEDULER_WORKLOAD_MANAGEMENT.md
# Scheduler workload management contracts

This document defines enterprise scheduling contracts for calendars, fairness, admission control, SLA policy, and workload simulation.

## Calendar and suppression controls

- `DagCalendar` defines timezone-aware blackout and holiday behavior.
- `BlackoutWindow` and `HolidayPolicy` encode operational suppression windows.
- `EnvironmentSuppression` allows environment-specific schedule suppression.

## Backfill orchestration and throttling

- `PartitionBackfillOrchestration` models partition-aware backfill planning.
- `BackfillThrottlingPolicy` reserves live capacity while limiting backfill submissions.
- `compute_partition_backfill_batches` and `apply_backfill_throttling` provide deterministic planning primitives.

## Fairness, service classes, and admission control

- `FairnessAlgorithm` and `StarvationPreventionPolicy` define anti-starvation controls.
- `ServiceClass` provides workload intent (`interactive`, `batch`, `archival`, `critical`).
- `QueueAdmissionPolicy` provides queue admission gates under resource pressure.

## Priority and batching behavior

- `WeightedPriorityPolicy` defines weighted priority scheduling with deterministic tie-breaks.
- `weighted_priority_tie_break_order` orders submissions by weight, then stable deterministic fields.
- `RunBatchPolicy` and `run_batches` define optional grouped run dispatch.
- `ConcurrencyScope` defines governance scope boundaries for limits.

## Trigger buffering, previews, and conflict detection

- `DependencyTriggerBufferPolicy` defines dedup/buffering controls for bursty upstream triggers.
- `materialize_next_runs` provides deterministic next-`N` schedule previews.
- `detect_cron_conflicts` detects concentrated cron windows.
- `deduplicate_trigger_events` defines duplicate trigger key handling.

## Overrides, suppression annotations, and SLA

- `ScheduleSuppressionAnnotation` and `ScheduleOverrideRecord` preserve operator/audit context.
- `SlaPolicy` defines expected start/finish and latency budgets.
- `SchedulerSlaMetrics` + `SchedulerAlertRule` support SLA miss and saturation alerting.

## Simulation and maturity tracking

- `SchedulingSimulationSuite` defines simulation intent and fixture sets.
- `CrossSchedulerCompatibility` tracks compatibility assumptions for HA/sharded futures.
- `SchedulerMaturityMatrix` tracks readiness: local-only, durable, multi-queue, backfill, HA.

Fixtures:
