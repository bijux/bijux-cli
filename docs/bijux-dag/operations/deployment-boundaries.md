---
title: Deployment Boundaries
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-04
---

# Deployment Boundaries

Deploying `bijux-dag` means preserving graph meaning, execution identity,
backend authority, and retained evidence across the environment where nodes
actually run. Copying the same graph file is insufficient when runtime,
adapter, storage, policy, inputs, or executable identity changes.

The stable `v0.4.0` execution lanes are local shell/container execution,
shared-filesystem SLURM, and Kubernetes Jobs for container nodes with a shared
persistent volume claim. Generic HPC, public remote workers, and a scheduler
service are not part of this release boundary.

```mermaid
flowchart TB
    graph["graph · inputs · policy"]
    controller["bijux-dag controller"]
    backend{"selected backend"}
    local["host process or local container"]
    slurm["sbatch · sacct<br/>shared run directory"]
    kube["Kubernetes Job<br/>shared persistent volume"]
    evidence["manifest · attempts · traces<br/>outputs · backend evidence"]
    verify["strict verification<br/>comparison or replay"]

    graph --> controller --> backend
    backend --> local --> evidence
    backend --> slurm --> evidence
    backend --> kube --> evidence
    evidence --> verify
```

Every worker must return through the runtime's acceptance path. Scheduler or
pod completion alone is not a successful node result.

## Deployment Identity

| Identity input | Why it matters | Evidence |
| --- | --- | --- |
| graph and plan | defines nodes, dependencies, triggers, declared effects, and execution intent | graph/plan fingerprints and retained source |
| tool and runtime | determines schema, lifecycle, retry, cache, and replay meaning | build identity and runtime metadata |
| adapter/backend | translates node work and terminal state | adapter identity, backend selection, batch evidence |
| inputs and environment | can change process behavior without changing graph text | declared input index and governed environment evidence |
| storage roots | determine where workers read inputs and publish retained results | run root, work paths, output index, storage metadata |
| policy | authorizes declared effects and execution controls | effective policy and refusal/acceptance evidence |
| artifacts | carry downstream data and replay authority | hashes, proofs, lineage, and strict verification |

## Backend Contracts

| Backend | Required deployment contract | Important limit |
| --- | --- | --- |
| local shell | trusted host code, required executables, writable governed roots | no host filesystem or network sandbox |
| local container | supported Docker or Podman engine, valid image and mount plan | isolation is limited to the engine and selected controls |
| SLURM | `sbatch` submission, `sacct` polling, and a run directory workers can reopen at the same usable location | only the shared-filesystem lane is supported; scheduler access is not created by the runtime |
| Kubernetes | `kubectl`, container nodes, volume claim, shared root, and Job pods mounting the same persistent storage contract | not a generic remote-worker service; cluster policy and image trust remain external |

Shell and container nodes are not equivalent security boundaries. SLURM and
Kubernetes are not equivalent storage or failure domains. Preserve the
selected backend and its effective configuration with every run used for
comparison or release evidence.

## Preflight By Lane

### Local

- identify the exact `bijux-dag` executable and version;
- validate the graph and required inputs;
- confirm host commands or container engine availability;
- keep run, cache, and output roots explicit;
- inspect effective isolation before executing untrusted code.

### Shared-filesystem SLURM

- prove the controller and scheduled worker can address the same retained run
  directory;
- verify `sbatch` and `sacct` identity, authentication, and terminal-state
  visibility;
- establish queue/partition policy and timeout expectations;
- preserve job identifiers, polling evidence, worker traces, and scheduler
  stderr;
- reject a deployment where a worker cannot reopen or finalize governed
  evidence.

### Kubernetes Jobs

- select a valid persistent volume claim and shared root;
- ensure controller and Job pods agree on the mounted run-directory mapping;
- verify namespace, service-account, image pull, resource, deadline, and pod
  log access outside the DAG runtime;
- retain Job identity, phase translation, container result, and finalized run
  evidence;
- treat a successful pod phase without accepted outputs and traces as
  incomplete.

## Cross-Environment Replay And Comparison

Replay verifies the source run before using it, and `--sandbox` prevents
writes to that source run. It does not recreate the original host, container
engine, scheduler, cluster, clock, network, or secrets.

When moving a run:

1. preserve the source run without in-place edits;
2. verify it before transfer;
3. transfer every referenced artifact and proof;
4. record the destination runtime, adapter, backend, and storage mapping;
5. replay into a new run directory;
6. compare with the mode that owns the question—semantic, artifact,
   provenance, timing, policy, cache, or raw.

A semantic match does not imply equal timing or equal isolation. A raw
difference does not automatically imply different workflow meaning.

## Failure Attribution

| Symptom | First owner |
| --- | --- |
| graph validates locally but not remotely | binary/schema identity and transferred source |
| worker never starts | scheduler/cluster submission and external authorization |
| worker starts but cannot load inputs | shared storage path or mount mapping |
| process succeeds but node fails | declared output, trace, or finalization contract |
| terminal state never arrives | backend polling or state translation |
| replay refuses source run | retained integrity, compatibility, or missing evidence |
| comparison changes only across backends | adapter, environment, provenance, or timing identity |
| isolation expectation fails | host/container/cluster control, not backend selection alone |

## Code Anchors

- `crates/bijux-dag-runtime/src/internal/control/runtime_controls.rs`
- `crates/bijux-dag-runtime/src/backend/runtime/container_execution.rs`
- `crates/bijux-dag-artifacts/src/storage/services.rs`
- `crates/bijux-dag-app/src/routes/replay_routes.rs`

## Next Reads

- [Support Matrix](../interfaces/support-matrix.md)
- [Observability And Diagnostics](observability-and-diagnostics.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
- [Release and Versioning](release-and-versioning.md)
- [Execution Security And Isolation](security-isolation-truth.md)
