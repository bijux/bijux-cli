---
title: Remote Execution Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Remote Execution Model

This contract separates shipped batch submission from modeled remote-worker
coordination. Both cross a process or host boundary, but they do not have the
same release status or controller semantics.

## Execution Mode Authority

| Mode | Release status | Owned boundary |
| --- | --- | --- |
| local shell | implemented | host process launched and accepted by the local controller |
| local container | implemented | Docker or Podman execution accepted by the local controller |
| Kubernetes | implemented Kubernetes Job backend | container node submitted through `kubectl` with a shared persistent volume claim |
| SLURM | implemented shared-filesystem SLURM backend | node submitted through `sbatch`, observed through `sacct`, and completed through a shared run directory |
| remote worker | modeled only | typed payload, lease, heartbeat, and result-handoff contracts |
| generic HPC | unreleased | no abstract scheduler or storage contract |

The public remote-worker service is not implemented.
The generic HPC backend is not implemented. Those limits do not downgrade the
bounded Kubernetes and SLURM lanes classified as stable in
`contracts/foundation/dag_release_truth_table.v1.json`.

## Scope

This model governs:

- remote execution identity
- typed worker request and result payloads
- input artifact integrity
- artifact and observability handoff declarations
- duplicate and late result handling expectations
- execution-mode classification

It does not define Kubernetes or SLURM request construction. Those shipped
lanes are governed by `docs/spec/BATCH_EXECUTION_MODEL.md` and their
backend-specific executable contracts.

## Worker Request Contract

A modeled remote-worker request carries:

- `run_id`, `node_id`, `attempt_id`, and `backend_id`
- the accepted graph and concrete node
- resolved node parameters
- worker run-root and optional cache-root paths
- execution policy and absolute-path settings
- planner contract version
- node, definition, environment, parameter, execution, evidence, and
  execution-contract fingerprints
- optional input artifacts with normalized relative paths, bytes, and matching
  SHA-256 digests

The worker rejects an input artifact when its declared digest does not match
the delivered bytes. It does not reinterpret graph policy or invent a new
execution identity.

## Modeled Worker Semantics

The modeled worker executes `const` and `shell` payloads. External adapter
kinds fail explicitly rather than appearing supported. Local container
execution is not routed through this worker model.

Worker execution returns `RemoteNodeExecutionResult`. Its `node_result` uses
the same durable `NodeResult` shape as local execution for status, streams,
outputs, attempts, evidence, failure details, and container metadata.
Schema parity does not make the remote-worker transport a shipped service.

## Identity And Handoff

Remote identity is immutable across request, status, and result evidence:

- run, node, attempt, and backend identifiers must agree
- artifact handoff declares upload and download endpoints plus an integrity
  requirement
- observability handoff declares stream behavior, trace forwarding, and a
  retention hint
- duplicate or late results remain observations until the controller accepts
  one result into retained run state

The controller remains authoritative for scheduling, retry, accepted outputs,
and terminal run state.

## Failure And Recovery Boundary

The model preserves launch, execution, integrity, and handoff failures through
the shared node-result contract. It does not currently promise:

- durable remote lease recovery after controller loss
- multi-controller ownership transfer
- a public worker registration or authentication protocol
- generic remote storage negotiation
- scheduler-independent HPC portability

These omissions are product limits, not invitation to infer behavior from
modeled types.

## Proof

- `crates/bijux-dag-runtime/tests/remote_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/distributed_contracts.rs`
- `crates/bijux-dag-runtime/tests/runtime_execution_module_entrypoints_contracts.rs`
- `crates/bijux-dev/src/commands/ops.rs`

An incompatible identity, payload, handoff, result, or mode-classification
change must update this contract and its executable proofs together.
