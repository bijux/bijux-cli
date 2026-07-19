---
title: Known Limitations
audience: operators
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Known Limitations

This register states what `bijux-dag v0.4.x` does not promise. A limitation is
not an implied roadmap commitment: the release target records the current
decision, and the planned fix states the evidence required before that decision
can change.

Use [Release Boundary](../foundation/release-boundary.md) to classify a surface
as stable, experimental, simulated, internal, or future-facing. Use this
register to assess operational consequences within that classification. The
[Bijux Dag Roadmap](../roadmap.md) records candidate release lanes rather than
current guarantees.

## Stable Local Execution Limitations

### LIM-007 Stable local execution remains a single-controller runtime

- stability class: `stable-surface`
- affected command or API: `bijux-dag run`, `bijux-dag replay`, and the local controller
- limitation: one process owns scheduling, cache decisions, and run-state mutation; retained local evidence supports inspection, not replicated controller recovery.
- impact: controller exit or host loss has no multi-controller failover or high-availability scheduler guarantee.
- workaround: supervise each local job externally, retain its run root, and use external schedulers only as submitters into the local `bijux-dag run` surface.
- planned fix: require implemented and tested multi-controller recovery, coordination durability, and an explicit release-boundary promotion.
- release target: replicated controller semantics are outside `v0.4.x`.

## Shell Isolation Limitations

### LIM-001 Shell policy denial is not a syscall sandbox

- stability class: `stable-surface`
- affected command or API: local shell execution through `run` and `replay`
- limitation: deny flags reject declared effects, while `--clean-env` only shapes environment bindings.
- impact: a command that omits an effect still executes as a host process unless another boundary contains it.
- workaround: run only trusted shell graphs and add host-level containment when process behavior is not trusted.
- planned fix: require a dedicated sandboxed executor and executable isolation contracts before widening the claim.
- release target: host-process network, clock, and arbitrary-filesystem isolation are not promised in `v0.4.x`.

### LIM-003 Clock denial does not virtualize time

- stability class: `stable-surface`
- affected command or API: shell and container execution with `--deny-clock`
- limitation: the flag rejects declared clock effects; it does not freeze, replace, or intercept wall-clock access.
- impact: tools can still observe ambient time when their execution boundary permits it.
- workaround: declare clock use honestly and provide an external deterministic time source where reproducibility requires one.
- planned fix: require an enforceable time source across every supported backend before claiming clock isolation.
- release target: wall-clock virtualization is outside `v0.4.x`.

## Container Limitations

### LIM-002 Container no-network enforcement depends on the runtime boundary

- stability class: `stable-surface`
- affected command or API: container execution and `runtime isolation`
- limitation: no-network enforcement is claimed only when the selected Docker or Podman boundary accepts the required engine flag.
- impact: the container path is stronger than shell declaration checks but is not equivalent to a virtual machine boundary.
- workaround: inspect `runtime isolation` and reject execution unless network denial is reported as engine-enforced.
- planned fix: widen the claim only for engines covered by enforcement and failure contracts.
- release target: network isolation remains engine-conditional throughout `v0.4.x`.

## Scheduling Limitations

### LIM-008 Internal schedule and backfill lanes are not stable scheduler APIs

- stability class: `stable-surface`
- affected command or API: internal `schedule` routes, backfill workflows, and `BIJUX_DAG_ENABLE_INTERNAL=1`
- limitation: preview, dispatch, ledger mutation, and backfill controls are maintainer proof surfaces, not visible stable operator commands.
- impact: internal automation can narrow or change within `v0.4.x` without violating the public CLI contract.
- workaround: submit stable `run` commands from an external scheduler; do not build production control paths on the internal namespace.
- planned fix: require defined persistence, compatibility, recovery, and operator lifecycle contracts before public promotion.
- release target: the schedule namespace remains internal throughout `v0.4.x`.

## Remote/Distributed Limitations

### LIM-009 Remote coordination and batch backends are modeled, not shipped

- stability class: `stable-surface`
- affected command or API: modeled remote-worker coordination, fake batch execution, generic HPC, and public scheduler-service surfaces
- limitation: remote-worker leases, heartbeats, payload handoff, and fake batch lifecycle are typed models or simulations. This limitation does not apply to the bounded `run --backend kubernetes` and shared-filesystem `run --backend slurm` lanes classified as stable by the release boundary.
- impact: modeled coordination evidence does not prove a public remote-worker service, generic HPC abstraction, or distributed scheduler.
- workaround: use the documented Kubernetes or SLURM backend only when its shared-storage prerequisites are satisfied; treat remote-worker and generic scheduler material as design evidence.
- planned fix: require production coordination semantics, recovery behavior, operator lifecycle documentation, support commitments, and end-to-end evidence before promoting any broader remote surface.
- release target: remote and distributed execution remain outside the stable `v0.4.x` product promise.

## API Stability Limitations

### LIM-005 Hidden experimental DAG routes are callable but not stable operator APIs

- stability class: `experimental-surface`
- affected command or API: explicit hidden routes including `graph-lint`, `why-cache-missed`, `export`, `import`, `fsck`, and `prove`
- limitation: these routes are callable but excluded from the visible `bijux-dag --help` compatibility contract.
- impact: automation using them can break within `v0.4.x` while the stable operator surface remains compatible.
- workaround: use visible commands and stable crate-root APIs for production; opt into `commands --lane experimental` only with explicit version control.
- planned fix: promote routes individually with documentation, compatibility commitments, and release tests, or keep them hidden.
- release target: experimental routes have no `v0.4.x` stability guarantee.

### LIM-006 Simulated platform-control namespaces remain repository-owned modeling surfaces

- stability class: `simulation-surface`
- affected command or API: hidden `control-plane`, `dataset`, `enterprise`, `fleet`, `federation`, `governance`, `incident`, and `lab` namespaces
- limitation: these namespaces require `BIJUX_DAG_ENABLE_SIMULATED=1` and model behavior that the production runtime does not ship.
- impact: command availability is not evidence of an enterprise control plane or distributed execution fabric.
- workaround: use simulated routes only for deliberate modeling and use the visible operator surface for real workflows.
- planned fix: retain the simulation boundary or implement real semantics, tests, support policy, and release documentation before promotion.
- release target: simulated namespaces remain non-public throughout `v0.4.x`.

## Cache/Replay Limitations

### LIM-004 Replay sandbox protects the source run directory only

- stability class: `stable-surface`
- affected command or API: `bijux-dag replay --sandbox`
- limitation: sandbox mode forbids writes to the source run directory; it does not isolate replay process syscalls.
- impact: the replayed process retains the network, clock, and filesystem reach of its execution backend.
- workaround: use `--sandbox` for evidence integrity and choose a stronger backend or host boundary for untrusted code.
- planned fix: keep evidence protection separate from process-isolation claims until a stronger executor ships.
- release target: source-run write protection is the complete `v0.4.x` replay sandbox guarantee.

Executable coverage for sandbox planning and source-evidence handling is in
`crates/bijux-dag-app/tests/replay_proof_contract.rs`. The complete enforcement
boundary and its runtime contract tests are mapped in [Execution Security And
Isolation](../operations/security-isolation-truth.md).

### LIM-010 Cache and replay proof depends on exact retained evidence

- stability class: `stable-surface`
- affected command or API: `cache verify`, `why-cache-missed`, `replay`, `export`, and `import`
- limitation: reuse requires matching execution, environment, input-lineage, adapter, contract, backend, and output-hash evidence; replay proves only what the retained run or export mode preserves.
- impact: cache entries do not promise cross-backend or cross-environment portability, and `manifest-only` or `without-artifacts` bundles cannot prove artifact-backed replay.
- workaround: retain the run directory or use `export --with-files`, then inspect cache verification, miss reasons, and output hashes before claiming equivalence.
- planned fix: require broader bundle, backend, and compatibility contracts before expanding portability claims.
- release target: exact local evidence remains the `v0.4.x` cache and replay proof boundary.

## Register Governance

- Every active record keeps a stable `LIM-` identifier and all seven fields.
- A record belongs under the release boundary that owns its operational impact.
- A planned fix describes proof required to change the claim; it is not a date
  promise.
- A resolved limitation is removed with the release evidence that closes it.
- Risks with uncertain outcomes belong in the [Risk Register](risk-register.md),
  not in this register.

## Next Reads

- [Risk Register](risk-register.md)
- [Execution Security And Isolation](../operations/security-isolation-truth.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
