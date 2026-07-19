# bijux-dag-runtime

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-runtime?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-runtime)
[![Rust docs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-dag-runtime](https://img.shields.io/crates/v/bijux-dag-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-dag-runtime) [![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag-artifacts](https://img.shields.io/crates/v/bijux-dag-artifacts?label=artifacts&logo=rust)](https://crates.io/crates/bijux-dag-artifacts) [![bijux-dag-core](https://img.shields.io/crates/v/bijux-dag-core?label=core&logo=rust)](https://crates.io/crates/bijux-dag-core) [![bijux-dag-app](https://img.shields.io/crates/v/bijux-dag-app?label=app&logo=rust)](https://crates.io/crates/bijux-dag-app) [![bijux-dag-cli](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag-cli](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-dag-runtime docs](https://img.shields.io/badge/docs-runtime-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/) [![bijux-dag-runtime docs.rs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag-artifacts docs.rs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts) [![bijux-dag-core docs.rs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core) [![bijux-dag-app docs.rs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app) [![bijux-dag-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-runtime` is the execution engine for `bijux-dag`. It handles
runtime planning, scheduling, adapter invocation boundaries, policy checks,
replay classification, cache behavior, and trace emission.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate is the
runtime layer that executes, records, replays, and verifies that promise.

## Release Status

- public crate on the `v0.4.0` DAG release line
- execution-time layer for the local-first controller
- stable local, Kubernetes Job, and shared-filesystem SLURM execution lanes
- modeled remote-worker and generic scheduler contracts that remain outside
  the public operator promise

## What It Provides

- execution planning and node orchestration
- policy evaluation and runtime diagnostics
- replay, diff, cache, and artifact integration behavior
- local shell and local container execution
- Kubernetes Job execution for container nodes through `kubectl` and a shared
  persistent volume claim
- SLURM submission through `sbatch`, terminal-state polling through `sacct`,
  and result collection from a shared run directory
- versioned external-adapter handshakes for repository-integrated executors
- container engine detection, mounted input and output layout, stdout/stderr
  capture, and retained container identity
- branch pruning, skipped-lane recording, trigger-rule evaluation, and replay
  equivalence over selected execution paths

Use this crate when you need to execute validated DAG graphs, replay retained
runs, enforce runtime policy, or integrate with Bijux execution behavior from
Rust.

## What It Does Not Own

- authoritative graph schema and validation rules
- top-level command parsing or output presentation
- release-governance and maintainer report composition

## Good Fit

- executing validated graphs from Rust
- replaying or comparing retained runs
- integrating with cache, adapter, and runtime policy behavior
- consuming the same execution semantics that power `bijux-dag`

## Runtime Identity Rules

- runtime manifests and provenance records stamp the crate package version
  directly from build metadata
- an optional Git short SHA may be appended at build time when the crate is
  compiled from a repository checkout or injected through
  `BIJUX_DAG_BUILD_GIT_SHA` for release-tree builds
- runtime execution does not shell out to `git` to discover version identity
- replay and cache identity therefore do not depend on the operator's current
  working directory or any unrelated Git repository around the binary

Use these rules when reviewing runtime fingerprint drift or provenance output.

## Integration Boundaries

| Boundary | Runtime commitment | Authority |
| --- | --- | --- |
| external adapters | descriptor handshake, explicit execution paths, typed failure information, and adapter-binary identity in cache evidence | [Adapter Contract](../../docs/spec/ADAPTER_CONTRACT.md) |
| Kubernetes | container nodes submitted as Jobs, shared-volume workspace mapping, resource and deadline mapping, pod-state mapping, logs, and retained batch evidence | [Batch Execution Model](../../docs/spec/BATCH_EXECUTION_MODEL.md) |
| SLURM | shared-filesystem jobs submitted with `sbatch`, polled with `sacct`, and collected into the retained node result | [Batch Execution Model](../../docs/spec/BATCH_EXECUTION_MODEL.md) |
| retained lifecycle evidence | terminal status, validated lifecycle transitions, per-attempt output, and bounded log summaries | [Run Evidence Layout](../../docs/bijux-dag/interfaces/run-evidence-layout.md) |
| subprocess cleanup | process-group termination on Unix and explicit best-effort behavior on other hosts | [Execution Security And Isolation](../../docs/bijux-dag/operations/security-isolation-truth.md) |
| replay and cache | identity-aware reuse, refusal evidence, and replay verification | [Reproducibility Model](../../docs/bijux-dag/interfaces/reproducibility-model.md) |

These boundaries are observable contracts, not claims of host isolation.
Shell execution is not a VM boundary. Kubernetes and SLURM support is bounded
by the documented shared-storage contracts; it does not imply a public remote
worker service, generic HPC abstraction, or durable scheduler control plane.

## Backend Selection

The application layer selects the backend through `run --backend` and supplies
backend-specific configuration. This crate owns execution after that selection.

| Backend | Accepted work | Required environment | Explicit non-guarantee |
| --- | --- | --- | --- |
| `local` | shell and local-container nodes | host process or configured container engine | no VM or syscall isolation |
| `kubernetes` | container nodes | `kubectl`, a shared persistent volume claim, and a host/cluster path mapping | no general Kubernetes workflow controller |
| `slurm` | nodes executable by the configured worker command | `sbatch`, `sacct`, and a run directory visible to controller and worker | no support for arbitrary HPC storage or scheduler semantics |

The controller remains authoritative for accepted run state and retained
evidence in all three lanes. Scheduler status is input to that state machine,
not a replacement for it. Controller restart recovery is not currently part
of the batch-lane promise.

## Public Rust Surface

- browse docs.rs through `bijux_dag_runtime::stable` for the long-lived
  runtime compatibility lane
- use `bijux_dag_runtime::prelude` for common planning and execution workflows
- use focused crate-root imports only when you already know the exact runtime
  item you need
- use `bijux_dag_runtime::simulated_platform` only for deliberate modeled-platform
  and control-plane evidence work
- Kubernetes and SLURM runtime configuration is available in the stable lane;
  backend-heavy compatibility helpers remain callable for repository-owned
  support work but stay hidden from the primary docs.rs lane
- use lane-scoped command discovery in `bijux-dag-app` or `bijux-dag-cli`
  when you need to inspect experimental, simulated, or internal runtime
  surfaces without widening the default operator contract

## Source Layout

- `src/runtime_core`: planning, execution, governance, and state transitions
- `src/adapters`: built-in and external adapter boundaries
- `src/backend/runtime`: local, container, Kubernetes, SLURM, and batch
  execution
- `src/backend/distributed`: modeled coordination and remote-worker contracts,
  not stable operator services
- `src/artifacts`, `src/cache`, `src/replay`, `src/policy`: core runtime
  behavior around persisted evidence and reuse
- `src/diagnostics`: runtime-facing diagnostic helpers

## Reach For Another Crate When

- you need deterministic graph truth before runtime side effects:
  `bijux-dag-core`
- you need command routing or output shaping:
  `bijux-dag-app`
- you need only persisted artifact helpers without execution policy:
  `bijux-dag-artifacts`

## Verify A Runtime Claim

| Claim | Repository-backed proof |
| --- | --- |
| container inputs, outputs, and engine identity | [Container Packaging Workflow](../../docs/bijux-dag/operations/container-packaging-workflow.md) |
| Kubernetes and shared-filesystem SLURM support boundary | [Execution Mode Responsibilities](../../docs/bijux-dag/architecture/execution-mode-responsibilities.md) |
| cache reuse, invalidation, corruption refusal, and miss explanation | [Cache Behavior Workflow](../../docs/bijux-dag/operations/cache-behavior-workflow.md) |
| graph, execution, cache, and replay identity | [Reproducibility Model](../../docs/bijux-dag/interfaces/reproducibility-model.md) |
| branch selection, skipped lanes, and replay stability | [Branching Bulletin Workflow](../../docs/bijux-dag/operations/branching-bulletin-workflow.md) |
| retry evidence and focused replay repair | [Compliance-Gated Bulletin Workflow](../../docs/bijux-dag/operations/compliance-gated-bulletin-workflow.md) |

Schedule and backfill flows remain internal workflow lanes in v0.4.x. Their
presence in repository evidence is not a public runtime commitment.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/)
