# Failure Taxonomy Contract

## Scope
This contract defines failure classes and recovery expectations for runtime, replay,
scheduler, adapter, and artifact integrity surfaces.

Authoritative code and tests:
- `crates/bijux-dag-runtime/src/runtime_core/governance/semantics.rs`
- `crates/bijux-dag-runtime/tests/runtime_failure_contracts.rs`
- `crates/bijux-dag-runtime/tests/runtime_recovery_contracts.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`

## Failure classes
- `timeout`: execution exceeded declared time limits
- `cancelled`: explicit cancellation reached terminal state
- `dependency_failure`: upstream dependency failed or was unavailable
- `policy_violation`: execution violated policy constraints
- `cache_invalid`: cached evidence invalid for reuse
- `artifact_corruption`: artifact payload/proof integrity violation
- `adapter_failure`: backend/runtime infrastructure failure not mapped above

## Operational grouping
- transient candidates: timeout, adapter failure, selected dependency failures
- permanent candidates: policy violation, artifact corruption, structural dependency failures
- advisory diagnostic classes: replay mismatch and backend capability mismatch

## Recovery expectations
- checkpoint without terminal completion requires recovery action
- partial artifact presence requires recovery action
- recovery classification must be explicit for interruption scenarios:
  - process interruption
  - scheduler interruption
  - event stream corruption
  - bundle import interruption
  - backend communication interruption

## Explainability requirement
Failure-oriented operator surfaces must remain machine-readable and stable:
- `dag why-rerun`
- `dag run-explain-failure`
- replay mismatch reason grouping

## Benchmark requirement
Failure handling claims require benchmark evidence for:
- classification overhead and drift
- failure injection workflows
- recovery decision latency

## Stability level
Stable governance contract for `v0.1` release truth surfaces.
