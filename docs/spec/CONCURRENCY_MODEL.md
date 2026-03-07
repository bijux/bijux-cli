# Concurrency Model

## Scope

This document defines concurrency guarantees for runtime scheduling, execution
coordination, artifact writes, cache access, and run finalization.

## Ownership Model

- Scheduler readiness state is owned by `SchedulerState`.
- Execution orchestration state is owned by runtime engine loop.
- Shared cross-worker counters and status maps are owned by runtime
  coordination primitives.
- Storage path mutation is centralized under run-dir and cache APIs.

## Shared Mutable State Inventory

| State | Owner | Synchronization | Notes |
| --- | --- | --- | --- |
| ready queue and indegree | `SchedulerState` | single-owner mutable state | deterministic updates |
| retry queue | `SchedulerState` | single-owner mutable state | explicit retry requeue |
| scheduler event log | `SchedulerState` | single-owner mutable state | ordered sequence IDs |
| run summary counters | runtime coordination | `Mutex` | monotonic count updates |
| trace write records | runtime coordination | `Mutex` | atomic append semantics |
| cache claim map | runtime coordination | `Mutex` | single fingerprint claim |
| latest-link update lock | runtime coordination | `Mutex` | prevents concurrent mutation races |

## Scheduling Concurrency Guarantees

- Concurrent predecessor completion can unlock a downstream node at most once.
- Retry requeue cannot duplicate node eligibility.
- Cancellation and timeout are terminal for scheduling decisions in a loop tick.
- Concurrency level tuning (`jobs`, `max_parallelism`) may alter throughput but
  not semantic node-set outcomes for deterministic plans.

## Artifact and Cache Concurrency Guarantees

- Trace append operations are serialized by coordination locks.
- Cache claims for a fingerprint are single-winner per in-memory coordination
  instance.
- Run summary updates are monotonic and merged under one lock.

## Unsafe Policy

- Runtime crate policy: no `unsafe` unless documented by an ADR and covered by
  dedicated tests.
- Control-plane audit reports every `unsafe` block and owner file.

## Stress and Flake Discipline

- Deterministic stress tests run medium graphs repeatedly under high
  concurrency.
- Any nondeterministic failure must be recorded in the concurrency flake ledger.

## Recovery and In-Progress Access

- Import/export against in-progress run directories is rejected unless explicitly
  supported with a contract update.
- Controller restart recovery semantics must be explicit; if unsupported, fail
  fast with a clear diagnostic.

## Verification Surfaces

- `crates/bijux-dag-runtime/tests/scheduler_contract.rs`
- `crates/bijux-dag-runtime/tests/concurrency_contracts.rs`
- `bijux-dev-dag repo run --domain governance` suites:
- `scheduler-invariants`
- `runtime-unsafe-audit`
- `concurrency-model`
