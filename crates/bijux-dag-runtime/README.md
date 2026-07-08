# bijux-dag-runtime

`bijux-dag-runtime` is the execution engine for `bijux-dag`. It handles
runtime planning, scheduling, adapter invocation boundaries, policy checks,
replay classification, cache behavior, and trace emission.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate is the
runtime layer that executes, records, replays, and verifies that promise.

## Release Status

- public crate on the `v0.4.0` DAG release line
- execution-time layer for the public local DAG product
- contains modeled platform support lanes, but those are not public operator
  promises by default

## What This Crate Owns

- execution planning and node orchestration
- policy evaluation and runtime diagnostics
- replay, diff, cache, and artifact integration behavior
- adapter boundaries for local shell, local container, and external execution
  backends
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

## External Adapter Boundary

- runtime discovery reads executable adapters from `BIJUX_DAG_ADAPTERS_DIR`
- `info --json` must emit descriptor JSON on stdout only; stderr during the
  handshake is treated as a protocol violation
- `execute` receives `--node-spec`, `--workdir`, `--outdir`, and
  `--failure-path`
- when an adapter exits nonzero and needs precise failure classification, it
  should write a `FailureInfo` JSON envelope to `--failure-path`
- the external adapter binary SHA-256 participates in node trace evidence and
  cache identity, so changing the adapter binary invalidates cached reuse even
  if the adapter keeps the same declared `id` and `version`

## Persisted Lifecycle Evidence

Node traces persist lifecycle evidence separately from terminal `status`.

- terminal `status` stays the coarse completion lane such as `success`,
  `failed`, `cached`, or `cancelled`
- process-backed nodes also record `exit_code` when the backend exposes one
- `lifecycle_state` records the final execution interpretation using the stable
  runtime vocabulary: `pending`, `ready`, `queued`, `running`, `succeeded`,
  `failed`, `skipped`, `cached`, `cancelled`, and `timed_out`
- `lifecycle_transitions` records the validated path through those states so a
  cache hit, timeout, cancellation, or queued-but-never-started node remains
  inspectable after the run finishes
- `stdout` and `stderr` trace fields record the retained terminal log path,
  byte size, and a bounded tail excerpt from the terminal attempt
- `nodes/<node_id>/attempts/<attempt>/stdout.log` and `stderr.log` preserve
  per-attempt copies when the runtime records attempt history
- runtime capture streams process output into spill files before it is copied
  into retained node evidence, so large logs stay on disk instead of being
  buffered fully in memory during execution

## Subprocess Cleanup Contract

- On Unix hosts, controlled shell, external, python, and container-engine
  commands run in a dedicated subprocess group.
- When a node times out or an operator cancellation arrives, the runtime sends
  `TERM` and then `KILL` to that subprocess group so child and grandchild
  helpers do not keep running after the node finishes.
- If the runtime has to fall back because group signaling cannot complete
  cleanly, it records that degradation in the captured node `stderr` instead of
  silently claiming full cleanup.
- On non-Unix hosts, subprocess termination remains best-effort leader
  termination rather than a full process-tree guarantee.

## Public Rust Surface

- browse docs.rs through `bijux_dag_runtime::stable` for the long-lived
  runtime compatibility lane
- use `bijux_dag_runtime::prelude` for common planning and execution workflows
- use focused crate-root imports only when you already know the exact runtime
  item you need
- use `bijux_dag_runtime::simulated_platform` only for deliberate modeled-platform
  and control-plane evidence work
- backend-heavy compatibility helpers remain callable for repository-owned
  support work, but stay hidden from the primary docs.rs lane
- use lane-scoped command discovery in `bijux-dag-app` or `bijux-dag-cli`
  when you need to inspect experimental, simulated, or internal runtime
  surfaces without widening the default operator contract

## Source Layout

- `src/runtime_core`: planning, execution, governance, and state transitions
- `src/adapters`: built-in and external adapter boundaries
- `src/backend`: local and distributed backend support
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

## Representative Workflow

For the repository-backed example that exercises mounted container inputs,
retained outputs, recorded image digest, and clear engine-unavailable failure
behavior, use
[Container Packaging Workflow](../../docs/bijux-dag/operations/guides/container-packaging-workflow.md).

For the repository-backed example that exercises full-workflow cache hits,
selective invalidation, corruption refusal, and proof-backed cache rejection on
one retained workflow family, use
[Cache Behavior Workflow](../../docs/bijux-dag/operations/guides/cache-behavior-workflow.md).

For the canonical explanation of how graph identity, plan identity,
execution identity, environment identity, artifact hashes, cache keys, and
replay bundles fit together, use
[Reproducibility Model](../../docs/bijux-dag/interfaces/reference/reproducibility-model.md).

For the repository-backed example that exercises branch decisions, join trigger
evaluation, skipped-lane evidence, and replay stability, use
[Branching Bulletin Workflow](../../docs/bijux-dag/operations/guides/branching-bulletin-workflow.md).

For the repository-backed example that exercises retry accounting, replay
boundary input rematerialization, and post-repair verification on a failed run,
use
[Compliance-Gated Bulletin Workflow](../../docs/bijux-dag/operations/guides/compliance-gated-bulletin-workflow.md).

For the repository-backed internal evidence lane that exercises cron preview,
deterministic schedule run ids, queue dispatch, explicit ledger completion,
and the handoff from scheduled submission into a retained DAG run, use
[Scheduled Catalog Refresh Workflow](../../docs/bijux-dag/operations/guides/scheduled-catalog-refresh-workflow.md).

For the repository-backed internal evidence lane that exercises deterministic
backfill fanout, aggregate summary reporting, retried partition state, and
explicit handoff from backfill requests into retained DAG runs, use
[Historical Catalog Backfill Workflow](../../docs/bijux-dag/operations/guides/historical-catalog-backfill-workflow.md).

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/)
