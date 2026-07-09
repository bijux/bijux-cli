---
title: Remote Execution Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Remote Execution Model

`bijux-dag` models remote execution identity and handoff contracts while
keeping the generic public remote-worker and HPC surfaces separate from the
implemented Kubernetes and SLURM batch lanes.

## Scope

This model covers remote execution identity fields, typed worker payloads,
artifact and observability handoff requirements, explicit execution-mode
status classification, and node-result payload parity exercised by
`crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`.

## Execution-mode status

- `local`: implemented
- `container`: implemented local execution mode
- `remote-worker`: simulated worker execution mode
- `kubernetes`: implemented Kubernetes Job execution mode for container nodes
- `slurm`: implemented shared-filesystem batch execution mode
- `hpc`: not implemented as a generic backend family

> Not implemented: production Kubernetes/HPC promotion as generic backend families
>
> Not implemented: production Kubernetes/HPC

Container execution is a local engine-mediated lane. It does not imply remote
workers, Kubernetes scheduling, or HPC submission.

## Remote worker payload schema

A modeled remote worker payload must carry:

- remote execution identity with `run_id`, `node_id`, `attempt_id`, and
  `backend_id`
- the graph and concrete node being executed
- resolved node params as JSON
- remote workspace paths for the worker-owned run directory root and optional
  cache directory
- policy and absolute-path execution settings
- planner contract version
- execution fingerprint set:
  `node_fingerprint`, `node_definition_fingerprint`,
  `declared_environment_fingerprint`, `params_fingerprint`,
  `execution_fingerprint`, `evidence_fingerprint`, and
  `execution_contract_fingerprint`
- optional input artifacts, each with a normalized relative path, payload
  bytes, and a SHA-256 digest that must match the bytes delivered to the
  worker

## Worker execution semantics

- the modeled worker currently executes `const` and `shell` payloads
- `container` remains a local engine-mediated lane, not a remote worker
  promise
- external adapter kinds are rejected explicitly instead of being treated as
  silently supported
- adapter execution faults are surfaced through the shared `NodeResult`
  failure shape rather than a separate remote-only error body

## Result surface parity

Remote worker execution returns a `RemoteNodeExecutionResult` envelope whose
`node_result` field is the same serialized `NodeResult` type local execution
produces. Remote and local lanes therefore share the same durable node-result
schema for status, logs, outputs, evidence, failure details, attempts, and
container metadata.

## Remote identity and handoff rules

- remote identity must include `run_id`, `node_id`, `attempt_id`, and
  `backend_id`
- artifact handoff must declare upload and download endpoints plus an
  integrity requirement
- observability handoff must declare stream mode, trace forwarding behavior,
  and a retention hint

## Maturity boundary

This document governs modeled remote-worker surfaces plus execution-mode
classification. It does not claim implemented public remote workers or generic
HPC execution in `v0.4.0`, and it does not treat local container runs as
remote execution.

## Related tests

- `crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/distributed_contracts.rs`
- `crates/bijux-dag-runtime/tests/runtime_execution_module_entrypoints_contracts.rs`

## Versioning and change policy

Any incompatible change to remote identity fields, handoff semantics, or
execution-mode classification must update this model and the linked runtime
tests in the same change.
