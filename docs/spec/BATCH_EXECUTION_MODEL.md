---
title: Batch Execution Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Batch Execution Model

`bijux-dag` models batch job metadata, retry progression, heartbeat freshness,
and a simulated batch backend without claiming implemented scheduler
integration.

## Scope

This model covers batch job metadata validation, retry-attempt shaping,
heartbeat staleness, duplicate status detection, cancellation behavior, and
execution-mode reporting exercised by:

- `crates/bijux-dag-runtime/tests/batch_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs`

## Batch metadata and lifecycle rules

- batch metadata must include scheduler identity plus run, node, and attempt
  identity
- retry attempts must produce a new attempt identifier and a later submission
  timestamp
- heartbeat freshness is evaluated against a caller-provided staleness window
- duplicate scheduler status delivery must be detectable
- cancel requests must append explicit cancellation lifecycle evidence

## Execution-mode boundary

- `local` is an implemented execution mode
- `fake-batch-backend` is simulated
- `slurm-backend` remains aspirational and is not implemented as a supported
  execution backend in `v0.4.0`
- restart recovery is not implemented for the batch model

## Related tests

- `crates/bijux-dag-runtime/tests/batch_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs`

## Versioning and change policy

Any incompatible change to batch metadata requirements, lifecycle event
semantics, execution-mode classification, or restart-recovery claims must
update this model and the linked runtime tests in the same change.
