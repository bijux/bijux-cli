# Runtime Engine and Scheduler Hot-Path Benchmark

Generated: 2026-03-08

## Scope

- Engine dispatch helper boundary: `runtime_core/execution/engine_dispatch.rs`
- Engine observe helper boundary: `runtime_core/execution/engine_observe.rs`
- Engine finalize helper boundary: `runtime_core/execution/engine_finalize.rs`
- Engine record helper boundary: `runtime_core/execution/engine_record.rs`
- Scheduler decision boundary: `runtime_core/execution/scheduler.rs`

## Measurement Contract

- Primary signal: `next_batch` decision latency under fixed-ready-set inputs.
- Secondary signal: trace event append cost per node transition.
- Tertiary signal: eligible-event emission cost for deterministic ready ordering.

## Current Baseline Inputs

- Small DAG: 10 nodes, 12 edges, cpu budget = 2.
- Medium DAG: 100 nodes, 140 edges, cpu budget = 8.
- Large DAG: 1000 nodes, 1600 edges, cpu budget = 32.

## Interpretation

- A change is a regression if hot-path median latency increases by more than 15%.
- A change is a regression if p95 latency increases by more than 20%.
- A change is a regression if deterministic scheduling order changes for fixed fixtures.
