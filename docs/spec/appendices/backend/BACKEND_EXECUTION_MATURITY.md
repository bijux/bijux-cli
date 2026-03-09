# Cluster and backend execution maturity contracts

This document defines contract maturity for local, Kubernetes, Slurm, and generic batch backends.

## Backend contract surfaces

- `KubernetesExecutorContractV2`: pod lifecycle, log flow, artifact flow, cancellation semantics
- `SlurmExecutorContract`: job submit/poll/cancel with result mapping
- `GenericBatchExecutorContract`: non-Kubernetes HPC/batch abstraction
- Normative Kubernetes semantics: `docs/spec/K8S_ADAPTER_CONTRACT.md`
- Normative HPC semantics: `docs/spec/HPC_ADAPTER_CONTRACT.md`

## Capability and placement

- typed backend capability descriptors cover CPU, memory, GPU, ephemeral storage, network class
- placement policy maps node requirements to backend capability classes deterministically

## Failure normalization and retries

- backend error codes map to stable runtime failure kinds with retryability metadata
- normalization prevents backend-specific leakage into user-facing failure semantics

## Image, logs, and artifact staging

- image resolution provenance records source identity and resolved digest
- log collection contract supports partial-log recovery on failure
- remote artifact staging protocols support non-shared-filesystem clusters

## Cleanup and maintenance

- backend cleanup guarantees define cancellation/failure cleanup obligations
- maintenance mode supports active/draining/maintenance states
- readiness probes gate scheduler admission during degradation

## Routing, quotas, and conformance

- queue-to-backend routing policy incorporates cost, trust, tenant, and latency classes
- backend quota/saturation metrics are first-class for scheduler and observability
- backend conformance suites define mandatory checks before support claims

## Replay compatibility and simulations

- cross-backend replay rules define when replay is safe across executors
- backend simulation fixtures without release-truth ownership are disallowed by evidence governance

## Production readiness checklist

Production readiness requires deterministic replay support, passed conformance suite, verified cleanup guarantees, and integrated observability for each backend class.

Kubernetes conformance and evidence linkage:

- `docs/reports/foundation/archive/K8S_CONFORMANCE_GATE_REPORT.md`
- `docs/reference/SUPPORT_AND_COMPATIBILITY_MATRICES.md`

HPC conformance and evidence linkage:

- `docs/reports/foundation/archive/HPC_CONFORMANCE_GATE_REPORT.md`
- `docs/reference/SUPPORT_AND_COMPATIBILITY_MATRICES.md`
