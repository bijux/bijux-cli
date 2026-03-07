# HPC Adapter Contract

## Scope

Defines the HPC adapter semantics for scheduler-backed batch execution contracts in bijux-dag.

## Queue and partition mapping

Node resource requests map deterministically into HPC queue/partition selection:

- explicit node queue/partition wins when present
- otherwise adapter defaults are used

## Walltime mapping

Node timeout maps to scheduler walltime as `HH:MM:SS`.

## Retry precedence

Retry ownership precedence is explicit:

1. scheduler-native retry policy
2. bijux retry policy
3. no retry

## Scratch and staging semantics

Each run/node pair uses deterministic scratch and staging directories:

- `/scratch/<run_id>/<node_id>`
- `/staging/<run_id>/<node_id>`

## Failure normalization

Required mappings:

- `SLURM_QUEUE_REJECTED` -> `configuration` (non-retryable)
- `SLURM_INVALID_ACCOUNT` -> `configuration` (non-retryable)
- `SLURM_WALLTIME_EXCEEDED` -> `timeout` (retryable)
- `SLURM_PREEMPTED` -> `infrastructure` (retryable)

## Polling, log collection, and cleanup

- Lost poll response recovery is explicit and timeout-bounded.
- Long-running jobs use chunked log collection semantics.
- Staged-input cleanup and scratch retention are explicit policy decisions.

## Array job and unsupported feature behavior

- Array job support is scheduler-specific (`slurm` supported by contract).
- Unsupported scheduler features must be rejected explicitly.

## Environment and scheduler identity capture

- Module/environment setup contributes to an environment fingerprint.
- Scheduler name/version capture is required in run metadata surfaces.

## Contract tests

- `crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs`
