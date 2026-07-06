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
and simulated batch backends. The repository now includes modeled Kubernetes
and SLURM lanes that execute through shared runtime adapters while keeping
cluster and scheduler semantics explicitly simulated rather than
control-plane-backed.

## Scope

This model covers batch job metadata validation, retry-attempt shaping,
heartbeat staleness, duplicate status detection, cancellation behavior,
execution-mode reporting, and the modeled Kubernetes and SLURM execution lanes
exercised by:

- `crates/bijux-dag-runtime/tests/batch_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs`
- `crates/bijux-dag-runtime/tests/kubernetes_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/kubernetes_backend_contracts.rs`
- `crates/bijux-dag-runtime/tests/slurm_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/slurm_backend_contracts.rs`

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
- `kubernetes` is a supported simulated execution backend that captures job
  identity, pod lifecycle, terminal phase mapping, workspace transfer
  contracts, and node logs through the shared runtime lane
- `kubernetes` is not a cluster-backed `v0.4.0` public operator promise
- `slurm-backend` is a supported simulated execution backend that captures job
  identity, scheduler lifecycle, terminal status mapping, and node logs through
  the shared runtime lane
- `slurm-backend` is not a cluster-backed `v0.4.0` public operator promise
- restart recovery is not implemented for the batch model

## Related tests

- `crates/bijux-dag-runtime/tests/batch_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs`
- `crates/bijux-dag-runtime/tests/kubernetes_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/kubernetes_backend_contracts.rs`
- `crates/bijux-dag-runtime/tests/slurm_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/slurm_backend_contracts.rs`

## Versioning and change policy

Any incompatible change to batch metadata requirements, lifecycle event
semantics, execution-mode classification, modeled Kubernetes or SLURM request
fields, or restart-recovery claims must update this model and the linked
runtime tests in the same change.
