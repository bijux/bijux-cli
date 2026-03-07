# Batch Execution Model

## Scope
This document defines the batch execution shape for long-running remote jobs.
In this repository, batch/HPC execution is modeled and simulated, not production
executed.

## Boundary decision
Batch/HPC support is modeled as an execution backend family.
It is not modeled as an adapter payload and not a control-plane-only wrapper.

## Required batch job metadata
- `scheduler_id`
- `submission_time_unix_ms`
- `run_id`
- `node_id`
- `attempt_id`
- `resource_request`
- `status_mapping`

## Retry semantics
- Retry submits a new scheduler job with a new `attempt_id`.
- Attempt lineage links all retry submissions to a single node execution lineage.

## Cancellation semantics
- Runtime cancellation maps to scheduler cancellation request.
- Cancellation outcome is recorded as success/failure/unknown delivery.

## Output and log collection
- stdout/stderr collection is mapped into run attempt observability.
- declared output collection must complete before attempt is finalized success.
- delayed artifact availability is represented as pending collection state.

## Remote failure mapping
- stale status updates -> transient remote state error
- missing status updates -> unknown remote state error
- duplicate delivery -> idempotent state application required

## Long-run progress model
- batch attempts emit heartbeat records while active.
- heartbeat timeout policy determines stale-attempt detection.

## Recovery boundary
Controller restart recovery for active remote batch attempts is not implemented as
fully resumable execution in this repository. Restart detection must fail
explicitly and report unsupported recovery.

## Mode classification
- implemented: local, subprocess
- simulated: batch contract and fake batch backend
- aspirational: production Slurm/PBS/Kubernetes execution backend

## Verifying tests
- `crates/bijux-dag-runtime/tests/batch_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs`
