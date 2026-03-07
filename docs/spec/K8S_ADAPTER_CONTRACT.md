# K8S Adapter Contract

## Scope

Defines the Kubernetes adapter semantics required for bijux-dag conformance. This contract governs semantic parity expectations with local execution and failure normalization into runtime taxonomy.

## Resource mapping

Node execution inputs map deterministically into Kubernetes resource requests and limits:

- CPU: `cpu_units * 1000` milliCPU request, `2x` milliCPU limit.
- Memory: `memory_mib` request, `1.5x` limit (minimum equal to request).

## Timeout mapping

- Node timeout maps to `activeDeadlineSeconds`.
- Timeout values are clamped to a minimum of `1` second.

## Retry mapping

- Node retry count maps to Job `backoffLimit`.
- Node retry backoff maps to adapter retry wait (`retry_backoff_seconds`).

## Cancellation mapping

- Node cancellation grace maps to pod/job `terminationGracePeriodSeconds`.
- Cancellation grace values are clamped to a minimum of `1` second.

## Equivalence expectations

Kubernetes and local outcomes are considered equivalent when all of the following match:

- DAG shape class.
- Node terminal statuses.
- Output artifact hashes.
- Cache-hit node set.
- Replayed node set.

Equivalence fixtures include:

- simple DAG
- fan-out DAG
- fan-in DAG
- cache-hit DAG
- partial replay DAG

## Failure normalization

Required mappings:

- `K8S_POD_EVICTED` -> `infrastructure` (retryable)
- `K8S_IMAGE_PULL_BACKOFF` -> `configuration` (non-retryable)
- `K8S_POD_PENDING_TIMEOUT` -> `infrastructure` (retryable)

## Secret/config injection

Adapter validation must reject execution when required secret/config references are missing.

## Log and artifact collection

- stdout/stderr capture must be equivalent for semantically equivalent runs.
- Artifact collection states are explicit: `complete`, `partial`, `missing`.

## Workdir volume semantics

- `emptyDir`: does not survive pod restart/reschedule.
- `persistentVolumeClaim`: survives pod restart/reschedule.

## Async watch event handling

Terminal completion recording must remain deterministic under out-of-order and duplicate watch events.
Terminal phases are `Succeeded`, `Failed`, `Cancelled`; reduction keeps the latest terminal event by sequence.

## Contract tests

- `crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs`
