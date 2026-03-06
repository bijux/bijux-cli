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

- `benchmarks/fixtures/scheduling/enterprise/load_spike.json`
- `benchmarks/fixtures/scheduling/enterprise/trigger_storm.json`
- `benchmarks/fixtures/scheduling/enterprise/mass_backfill.json`
