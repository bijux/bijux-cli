# Scheduler MVP contract

## Scope

The scheduler MVP defines trigger evaluation, schedule validation, run submission ordering, and queue isolation semantics for the local runtime.

## Minimum supported trigger types

- `manual`
- `cron`
- `event`
- `dependency`
- `signal`
- `backfill`

## Core guarantees

- Schedule definitions are validated before submission.
- Invalid policy combinations are rejected during validation, not at dispatch time.
- Runs emitted on the same scheduler tick are ordered deterministically by `(created_unix_ms, schedule_id, run_id)`.
- Schedule definition and execution submission are separate contracts.
- Schedule state persistence supports `pending`, `running`, and `completed` submission states.
- Schedule evaluation emits an audit record with decision and reason.

## Catch-up and backfill behavior

- Catch-up is explicit and bounded by `max_catch_up_runs`.
- Backfill requires queue concurrency caps.
- Backfill without a non-zero `max_parallelism` is invalid.

## Dry-run support

- `bijux-dev-dag schedule validate` validates schedule registry structure and trigger syntax.
- `bijux-dev-dag schedule preview` outputs preview fire-time signals for each schedule.

## Fixture set

Scheduling simulation fixtures live in:

- `benchmarks/fixtures/scheduling/cron_storm.json`
- `benchmarks/fixtures/scheduling/backfill_saturation.json`
- `benchmarks/fixtures/scheduling/concurrency_pressure.json`
