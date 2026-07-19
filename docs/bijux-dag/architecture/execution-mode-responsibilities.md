---
title: Execution Modes and Coordination Boundaries
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Execution Modes and Coordination Boundaries

`bijux-dag` has one authoritative controller model across its implemented
execution lanes. Execution substrates may prepare and run work, but they do
not own accepted run state. This distinction matters most for Kubernetes,
SLURM, and the modeled remote-worker contracts, where scheduler or worker
events can arrive late, twice, or after controller recovery.

See [Known Limitations](../quality/known-limitations.md) before treating a
mode's contract coverage as a production-readiness claim.

## Mode Status

| Mode | Current status | Execution boundary |
| --- | --- | --- |
| local | implemented core runtime | the controller dispatches work to the local worker pool |
| container | implemented local lane | the engine mediates mounts, environment, process execution, and declared outputs |
| Kubernetes | implemented batch backend for container nodes | Kubernetes Jobs run against a configured shared workspace |
| SLURM | implemented shared-filesystem batch backend | `sbatch` submits work and `sacct` supplies scheduler status |
| fake batch | simulation and contract proof | no external scheduler is contacted |
| remote worker | modeled contract, not a stable operator lane | typed payload, identity, result, and handoff behavior are exercised in tests |
| generic HPC | not implemented | SLURM support does not imply an abstract HPC backend family |

Implemented means that repository code and tests own the lane. It does not
remove deployment prerequisites or promote the lane beyond the support and
release boundaries stated elsewhere.

## Authority Split

The controller owns:

- dispatch identity and attempt lineage
- scheduling, retry, and terminal-state decisions
- acceptance of backend results and declared outputs
- mutation of the retained run record
- the evidence used by inspect, replay, and failure analysis

An execution backend owns:

- substrate capability reporting
- command or job preparation and launch
- in-flight status, logs, and scheduler identity
- provisional outputs and cleanup
- backend-specific error classification

A backend event is therefore an observation, not authoritative run state.
Artifacts become visible as accepted evidence only after controller validation
and durable record commit. This prevents a worker's partial success or a
duplicate terminal event from outranking the retained run record.

## Local and Batch Differences

Local execution keeps controller and worker lifecycle in one runtime process.
Kubernetes and SLURM preserve the same node-result and retained-evidence
contracts but add scheduler identity, submission, status reconciliation, and
shared-storage requirements.

The batch lanes do not currently promise controller restart recovery. The fake
batch lane proves metadata and lifecycle rules only; it is not a scheduler
service. Kubernetes requires `kubectl`, a shared persistent volume claim, and
a compatible controller run root. SLURM requires `sbatch`, `sacct`, and a run
directory visible to the scheduled worker.

## Remote Coordination Boundary

The shipped runtime remains single-controller. Remote payloads, heartbeats,
leases, status streams, log handoff, and duplicate-event reconciliation are
typed contracts used to test future coordination boundaries. They do not
constitute a public remote-worker service or distributed scheduler.

When a document mentions remote or distributed execution, read it as a
governed model unless the release boundary and support matrix explicitly
classify that operator surface as implemented and supported.

## Contract and Proof Map

| Concern | Governing contract | Primary proof |
| --- | --- | --- |
| container isolation | `docs/spec/CONTAINER_EXECUTION_CONTRACT.md` | `container_execution_contracts.rs` |
| remote payload and handoff | `docs/spec/REMOTE_EXECUTION_MODEL.md` | `remote_execution_contracts.rs` |
| batch metadata and lifecycle | `docs/spec/BATCH_EXECUTION_MODEL.md` | `batch_execution_contracts.rs` and backend-specific suites |
| controller event authority | `docs/spec/DISTRIBUTED_COORDINATION_MODEL.md` | `distributed_event_reconciliation_contracts.rs` |

Operations guidance owns deployment prerequisites. The
[Support Matrix](../interfaces/support-matrix.md) and
[Release Boundary](../foundation/release-boundary.md) own public stability
claims. Changing a mode's maturity, authority split, or recovery guarantee
requires those surfaces, the governing spec, and executable proof to change
together.
