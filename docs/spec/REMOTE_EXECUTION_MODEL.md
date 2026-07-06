---
title: Remote Execution Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Remote Execution Model

`bijux-dag` models remote execution identity and handoff contracts without
claiming implemented production backends for Kubernetes or HPC.

## Scope

This model covers remote execution identity fields, artifact and observability
handoff requirements, and explicit execution-mode status classification
exercised by `crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`.

## Execution-mode status

- `local`: implemented
- `container`: implemented local execution mode
- `kubernetes`: not implemented
- `hpc`: not implemented

> Not implemented: production Kubernetes/HPC

Container execution is a local engine-mediated lane. It does not imply remote
workers, Kubernetes scheduling, or HPC submission.

## Remote identity and handoff rules

- remote identity must include `run_id`, `node_id`, `attempt_id`, and
  `backend_id`
- artifact handoff must declare upload and download endpoints plus an
  integrity requirement
- observability handoff must declare stream mode, trace forwarding behavior,
  and a retention hint

## Maturity boundary

This document governs modeled remote execution surfaces only. It does not claim
implemented remote workers, production Kubernetes execution, or production HPC
execution in `v0.4.0`, and it does not treat local container runs as remote
execution.

## Related tests

- `crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`

## Versioning and change policy

Any incompatible change to remote identity fields, handoff semantics, or
execution-mode classification must update this model and the linked runtime
tests in the same change.
