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
