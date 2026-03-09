# BACKEND AND ADAPTER CONTRACTS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/ADAPTER_CONTRACT.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/ADAPTER_CONTRACT.md](./appendices/backend/ADAPTER_CONTRACT.md)

## SOURCE: docs/spec/ADAPTER_INTERFACE_SPEC_V0.1.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/ADAPTER_INTERFACE_SPEC_V0.1.md](./appendices/backend/ADAPTER_INTERFACE_SPEC_V0.1.md)

## SOURCE: docs/spec/ADAPTER_PLACEMENT.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/ADAPTER_PLACEMENT.md](./appendices/backend/ADAPTER_PLACEMENT.md)

## SOURCE: docs/spec/ADAPTER_RUNTIME_CONTRACT_V0.1.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/ADAPTER_RUNTIME_CONTRACT_V0.1.md](./appendices/backend/ADAPTER_RUNTIME_CONTRACT_V0.1.md)

## SOURCE: docs/spec/ATLAS_EXECUTION_CONTRACT.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/ATLAS_EXECUTION_CONTRACT.md](./appendices/backend/ATLAS_EXECUTION_CONTRACT.md)

## SOURCE: docs/spec/BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md
# Backend and adapter runtime contract

**What this spec is not**: benchmark strategy, scheduler tuning, or high-level governance philosophy.

## Scope

This contract is the canonical source for:

- runtime backend lifecycle and protocol expectations
- adapter identity, capability surfaces, and execution behavior
- backend equivalence and portability semantics
- backend placement and conformance boundaries

## Contract boundaries

- `BACKEND_CONTRACT`, `ADAPTER_CONTRACT`, and adapter interface specs are normative together.
- Backend-specific contracts for `k8s` and `hpc` are appendices to this cluster.
- Adapter semantics must not redefine run/graph meaning.

## Core requirements

- Backend lifecycle is explicit and deterministic in `prepare`, `launch`, `observe`, `finalize`, `cleanup`.
- Adapter identity and capabilities are stable and versioned.
- Conformance requires deterministic execution, error normalization, and cleanup guarantees.
- Cross-backend replay and equivalence behavior must be explicitly classified (`equivalent`, `fidelity-preserving`, `downgraded`).

## Evidence and implementation links

- Runtime enforcement: `crates/bijux-dag-runtime`
- Evidence and governance: `crates/bijux-dev-dag` backend conformance suites
- Canonical schemas/fixtures in backend and adapter test registries.

## Canonical appendices

- [backend contract](./appendices/backend/BACKEND_CONTRACT.md)
- [adapter contract](./appendices/backend/ADAPTER_CONTRACT.md)
- [adapter interface](./appendices/backend/ADAPTER_INTERFACE_SPEC_V0.1.md)
- [backend protocol stability](./appendices/backend/BACKEND_PROTOCOL_STABILITY_CONTRACT.md)
- [adapter runtime contract](./appendices/backend/ADAPTER_RUNTIME_CONTRACT_V0.1.md)
- [adapter placement and boundaries](./appendices/backend/ADAPTER_PLACEMENT.md)
- [backend execution maturity](./appendices/backend/BACKEND_EXECUTION_MATURITY.md)
- [backend equivalence](./appendices/backend/BACKEND_EQUIVALENCE_CONTRACT.md)
- [backend meaning boundary doctrine](./appendices/backend/BACKEND_MEANING_BOUNDARY_DOCTRINE.md)
- [kubernetes adapter](./appendices/backend/K8S_ADAPTER_CONTRACT.md)
- [hpc adapter](./appendices/backend/HPC_ADAPTER_CONTRACT.md)
- [atlas execution](./appendices/backend/ATLAS_EXECUTION_CONTRACT.md)

## SOURCE: docs/spec/BACKEND_CONTRACT.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/BACKEND_CONTRACT.md](./appendices/backend/BACKEND_CONTRACT.md)

## SOURCE: docs/spec/BACKEND_EQUIVALENCE_CONTRACT.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/BACKEND_EQUIVALENCE_CONTRACT.md](./appendices/backend/BACKEND_EQUIVALENCE_CONTRACT.md)

## SOURCE: docs/spec/BACKEND_EXECUTION_MATURITY.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/BACKEND_EXECUTION_MATURITY.md](./appendices/backend/BACKEND_EXECUTION_MATURITY.md)

## SOURCE: docs/spec/BACKEND_MEANING_BOUNDARY_DOCTRINE.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/BACKEND_MEANING_BOUNDARY_DOCTRINE.md](./appendices/backend/BACKEND_MEANING_BOUNDARY_DOCTRINE.md)

## SOURCE: docs/spec/BACKEND_PROTOCOL_STABILITY_CONTRACT.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/BACKEND_PROTOCOL_STABILITY_CONTRACT.md](./appendices/backend/BACKEND_PROTOCOL_STABILITY_CONTRACT.md)

## SOURCE: docs/spec/HPC_ADAPTER_CONTRACT.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/HPC_ADAPTER_CONTRACT.md](./appendices/backend/HPC_ADAPTER_CONTRACT.md)

## SOURCE: docs/spec/K8S_ADAPTER_CONTRACT.md
# Superseded by backend cluster contract

- Superseded by: [BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md](./BACKEND_AND_ADAPTER_RUNTIME_CONTRACT.md)
- Appendix source: [appendices/backend/K8S_ADAPTER_CONTRACT.md](./appendices/backend/K8S_ADAPTER_CONTRACT.md)

## SOURCE: docs/spec/appendices/backend/ADAPTER_CONTRACT.md
# Adapter contract

Adapter implementations must satisfy all requirements below.

## Scope
Defines adapter identity, capability metadata, execution behavior, and conformance requirements for built-in and external adapters.

## Identity and capabilities
- Stable adapter ID and version.
- Type-level origin classification: `BuiltIn` or `External`.
- Declared supported node kinds.
- Declared required effects.
- Declared output schema version.

## Execution contract
- Inputs are materialized only from declared upstream dependencies.
- Outputs must be declared and indexed; undeclared output writes are failures.
- Failures must be classified with stable machine codes.
- stdout/stderr capture must be persisted deterministically.
- Timeout and cancellation behavior must map to explicit runtime status.

## Environment contract
- Environment exposure is deny-by-default.
- Allowed environment variables must be explicit.
- Hermetic mode forbids undeclared environment and network access.

## Conformance
Every adapter must pass the runtime adapter conformance suite and metadata reproducibility checks across run and replay.

## Related tests
- `crates/bijux-dag-runtime/tests/adapter_conformance.rs`
- `crates/bijux-dag-runtime/tests/adapter_metadata_stability.rs`
- `tests/e2e/container/*`

## Versioning and change policy
Adapter contract changes must preserve existing descriptors or introduce explicit compatibility notes and conformance updates in the same change.

## SOURCE: docs/spec/appendices/backend/ADAPTER_INTERFACE_SPEC_V0.1.md
# Adapter Interface Spec v0.1

## Purpose

Define the stable runtime adapter interface and invariants for built-in and external adapters.

## Adapter Identity

- `adapter_id` MUST be non-empty and stable.
- `adapter_version` MUST be non-empty and semantic-version-like.
- Adapter identity tuple is `adapter_id@adapter_version`.

## Required Interface Fields

- supported node kinds
- required effect set
- produced output schema version
- execution entrypoint that returns structured node results

## Lifecycle Expectations

1. descriptor validation before execution
2. execution with declared effects only
3. structured status and error normalization
4. deterministic metadata persistence for replay/verify surfaces

## Compatibility Rules

- duplicate adapter identity tuples are disallowed
- identity/version changes are treated as compatibility-relevant for replay/cache
- adapter metadata does not alter canonical graph identity

## Conformance

Adapter implementations must pass runtime adapter conformance and registry capability contracts.

## SOURCE: docs/spec/appendices/backend/ADAPTER_PLACEMENT.md
# Adapter placement and boundary

## Decision
Built-in adapters remain in `bijux-dag-runtime` for now.

## Required boundary
- Runtime must expose adapter contracts through `runtime::adapter_api`.
- Adapter-specific implementation details must stay in adapter-focused modules.
- Cross-module execution logic must consume adapter contracts, not adapter-specific concrete details.

## Future option
A dedicated `bijux-dag-adapters` crate remains a valid future extraction once adapter contracts and runtime layering are fully stable.

## SOURCE: docs/spec/appendices/backend/ADAPTER_RUNTIME_CONTRACT_V0.1.md
# Adapter Runtime Contract v0.1

## Scope

Defines runtime behavior requirements for adapter execution, failure handling, and reproducibility.

## Runtime Guarantees

- lifecycle transitions are explicit and classified
- adapter errors propagate with stable machine-meaningful classes
- cancellation and timeout semantics are normalized
- execution metadata persists across run, export/import, and replay paths

## Backend Capability Query

Capability query output must be available for stable backend names:

- `local`
- `kubernetes`
- `hpc`
- `remote`

## Determinism and Concurrency

- registry dumps and capability payloads are deterministic for same inputs
- adapter execution behavior remains deterministic under supported concurrency settings

## Stress and Recovery

- adapter conformance includes stress-oriented execution surfaces
- failure-recovery paths remain machine-readable and non-panicking

## Evidence and Regression

- backend capability matrix and adapter scope reports are required generated artifacts
- adapter regression corpus fixtures track compatibility-sensitive scenarios

## SOURCE: docs/spec/appendices/backend/ATLAS_EXECUTION_CONTRACT.md
# Atlas Execution Contract

## Purpose
Define allowed `bijux-atlas` extensions without redefining graph meaning.

## Atlas may extend
- deployment and environment metadata
- scheduling hints
- operator dashboards and analytics

## Atlas may not redefine
- graph/run/artifact identity semantics
- replay equivalence meaning
- capability downgrade classification terms

## SOURCE: docs/spec/appendices/backend/BACKEND_CONTRACT.md
# Backend contract

## Scope

This contract is the normative source for execution backend lifecycle, capability binding, and conformance requirements.

## Lifecycle

Each backend implementation must implement this lifecycle in order:

1. `prepare`
2. `launch`
3. `observe`
4. `finalize`
5. `cleanup`

`cleanup` must execute after both successful and failed lifecycle paths.

## Backend API boundary

- `ExecutionBackend` is the only runtime backend interface.
- Runtime orchestration must call lifecycle hooks through this interface only.
- Backend binding must fail before execution when required capabilities are incompatible.

## Backend classes

- `FakeBackend`: deterministic backend for engine tests.
- `ProcessLikeBackend`: local subprocess-like backend model.
- Container and remote models are separate contracts and are not mixed into local process lifecycle.

## Capability descriptor

Every backend reports stable capability fields:

- `backend_name`
- `kind`
- `supports_env_shaping`
- `supports_timeout`
- `supports_stream_capture`

## Conformance requirements

Required conformance coverage includes:

- fake/process parity behavior
- prepare failure classification
- launch failure classification
- observe timeout classification
- cleanup on success
- cleanup on failure
- explicit environment shaping behavior
- undeclared output rejection
- registry report coverage

## Governance

- `bijux-dev-dag repo --domain governance --suite backend-contract` is required for backend changes.
- New backend implementations are blocked until backend contract conformance remains explicit and passing.

## SOURCE: docs/spec/appendices/backend/BACKEND_EQUIVALENCE_CONTRACT.md
# Backend Equivalence Contract

## Purpose
Define backend equivalence semantics and downgrade behavior across local, kubernetes, hpc, and remote targets.

## Equivalence classes
- `equivalent`: run outcomes and artifact lineage are semantically equivalent.
- `fidelity-preserving`: semantics preserved with advisory differences that do not change graph meaning.
- `downgraded`: semantics cannot be preserved fully; downgrade must be explicit.

## Required behavior
- Unsupported backend semantics must be rejected, not approximated.
- Backend-specific metadata must not alter graph identity.
- Backend-specific environment/runtime metadata may affect run identity where declared.
- Cross-backend replay and diff must emit explicit downgrade reasons.

## Operator surfaces
- `bijux dag capabilities --backend <name> --json`
- `bijux dag semantic-portability --backend <name> --json`
- `bijux dag equivalence-proof <run-a> <run-b> --backend-a <name> --backend-b <name> --json`

## SOURCE: docs/spec/appendices/backend/BACKEND_EXECUTION_MATURITY.md
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

## SOURCE: docs/spec/appendices/backend/BACKEND_MEANING_BOUNDARY_DOCTRINE.md
# Backend Meaning Boundary Doctrine

## Rule

Backends execute semantics; they must not redefine graph/run/artifact meaning.

## Invariants

- Canonical identity and lineage rules live in core/runtime contracts, not backend-specific reinterpretation.
- Backend capability gaps must be surfaced as capability states, not silent semantic drift.
- Unsupported backends must remain clearly labeled as modeled/simulated/unsupported.

## Enforcement

- Support policy in `docs/reference/EXECUTION_SUPPORT_POLICY.md`.
- Runtime and evidence conformance suites in `bijux-dev-dag`.

## SOURCE: docs/spec/appendices/backend/BACKEND_PROTOCOL_STABILITY_CONTRACT.md
# Backend Protocol Stability Contract

## Purpose

Define required backend protocol guarantees for handshake stability, version negotiation, compatibility, ordering, integrity, resilience, telemetry, and anomaly detection.

## Required protocol coverage

- protocol handshake and version negotiation behavior
- compatibility and error propagation behavior
- timeout and retry behavior
- message ordering and replay safety behavior
- corruption detection and serialization schema stability
- stress, latency, and resilience benchmark coverage
- telemetry, determinism, fuzzing, and anomaly detection coverage

## Required governance artifacts

- backend protocol regression corpus
- backend protocol verification suite
- backend protocol benchmark report
- backend protocol telemetry report
- backend protocol anomaly report
- backend protocol coverage report

## SOURCE: docs/spec/appendices/backend/HPC_ADAPTER_CONTRACT.md
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

## Universal vs scheduler-specific semantics

Universal (must hold for any supported HPC scheduler contract):

- queue/partition mapping determinism
- walltime mapping determinism
- explicit retry ownership precedence
- explicit failure normalization into runtime taxonomy

Scheduler-specific (allowed variation by scheduler family):

- array-job support behavior
- polling cadence and scheduler event delivery
- queue/account naming conventions

## Contract tests

- `crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs`

## SOURCE: docs/spec/appendices/backend/K8S_ADAPTER_CONTRACT.md
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

## Supported and out-of-scope surfaces

Supported contract surfaces in this repository:

- deterministic mapping for resources, timeout, retries, cancellation
- failure normalization for pod eviction, image pull backoff, pending timeout
- deterministic watch-event reduction and reconnect reconciliation
- capability declaration for node selector and affinity support

Out of scope in this repository (must remain explicitly non-claimable):

- production-grade Kubernetes execution backend
- live cluster watch stream integration
- PVC lifecycle provisioning and cleanup orchestration

## Intentionally rejected approximations

The adapter contract rejects unsafe approximation of Kubernetes-only fields that would
change execution meaning without explicit support:

- `hostNetwork`
- `hostPID`
- `privileged`
- `hostPath`
- `runtimeClassName`
