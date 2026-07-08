---
title: bijux-dag-runtime Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-07
---

# bijux-dag-runtime

`bijux-dag-runtime` owns execution-time behavior for DAG runs: planning,
scheduler policy, adapter invocation boundaries, artifact writing, replay,
cache policy, and runtime diagnostics.

At the product level, `bijux-dag` v0.4.0 is a local-first DAG runtime for
reproducible workflows with explicit graph contracts, deterministic execution
records, verified artifacts, cache explanation, and replayable run bundles.
This package is the runtime layer that executes, records, replays, and
verifies that promise.

Use this page when the question is about what happens after a graph has already
been accepted as valid.

The intended Rust import lanes are the crate root, `stable`, and `prelude`.
At the module level, the public lanes are intentionally limited to `stable`,
`prelude`, `experimental`, and `simulated_platform`. Hidden compatibility
helpers remain available for repository-owned coverage, and the `experimental`
lane is opt-in behind `experimental-public-api`.

## Reach For This Crate When

- a valid graph runs, retries, replays, or caches incorrectly
- runtime policy, capability checks, or trigger-rule behavior look wrong
- you need the execution semantics that power retained run evidence
- you are deciding whether a defect belongs to runtime policy or to CLI
  presentation

## What It Owns

| Surface | Ownership |
| --- | --- |
| execution engine | planning, scheduler behavior, backend invocation, replay semantics |
| runtime policy | policy evaluation, trace emission, error classification, capability checks |
| local container execution | engine detection, mounted input and output layout, stdout/stderr capture, and recorded image identity |
| branch execution | selected-lane pruning, skipped-node recording, trigger-rule evaluation, and replay equivalence checks |
| runtime artifacts | manifests, verification, cache lineage, and proof material |
| runtime identity | build-stamped version identity and deterministic runtime fingerprints |
| boundary | does not own authoritative DAG schema or user-facing CLI routing |

## What It Does Not Own

- graph schema authority and deterministic compilation belong to
  [`bijux-dag-core`](bijux-dag-core.md)
- command routing and response shaping belong to [`bijux-dag-app`](bijux-dag-app.md)
- package release policy and maintainer reports belong to repository and
  maintainer surfaces rather than this runtime crate

## Identity Guarantees

- runtime manifests and provenance records derive `tool_version` from the crate
  build, not from the operator's current shell environment
- a Git short SHA may appear only when it was captured during the build itself
  or injected through `BIJUX_DAG_BUILD_GIT_SHA` for a clean release tree
- runtime fingerprints stay stable when the same binary is executed from a
  different working directory
- unrelated ambient Git repositories are not allowed to rewrite replay or cache
  identity

## Lifecycle Evidence Contract

- node traces keep coarse terminal `status` separate from `lifecycle_state`
- the stable lifecycle vocabulary is `pending`, `ready`, `queued`, `running`,
  `succeeded`, `failed`, `skipped`, `cached`, `cancelled`, and `timed_out`
- `queued` is the scheduler handoff state after dependency readiness and before
  adapter execution starts
- `lifecycle_transitions` records the validated path through those states so
  cached reuse, timeout, cancellation, and pre-start failure paths remain
  inspectable after the run completes

## External Adapter Contract

- executable discovery reads from `BIJUX_DAG_ADAPTERS_DIR`
- `info --json` must emit descriptor JSON on stdout only; handshake stderr is a
  protocol violation
- `execute` receives `--node-spec`, `--workdir`, `--outdir`, and
  `--failure-path`
- nonzero adapter exits should write a `FailureInfo` JSON envelope to
  `--failure-path` when they need precise runtime failure classification
- the external adapter binary SHA-256 participates in cache identity and node
  trace evidence, so a binary change invalidates cached reuse even if the
  adapter keeps the same declared identity

## Source Layout

- `crates/bijux-dag-runtime/src/runtime_core`
- `crates/bijux-dag-runtime/src/adapters`
- `crates/bijux-dag-runtime/src/backend`
- `crates/bijux-dag-runtime/src/artifacts`
- `crates/bijux-dag-runtime/src/cache`
- `crates/bijux-dag-runtime/src/policy`
- `crates/bijux-dag-runtime/src/replay`
- `crates/bijux-dag-runtime/src/diagnostics`

## Practical Starting Points

- open the [DAG Handbook](../index.md) for the full DAG system map
- open [`bijux-dag-core`](bijux-dag-core.md) for graph truth and planning
  inputs
- open [`bijux-dag-app`](bijux-dag-app.md) for command orchestration and
  response shaping
- open [Reproducibility Model](../interfaces/reference/reproducibility-model.md)
  for the canonical explanation of plan identity, execution identity,
  environment identity, cache keys, and replay-bundle boundaries
- open [Cache Behavior Workflow](../operations/guides/cache-behavior-workflow.md)
  when you want a real execution path for cache hits, invalidation, corruption
  refusal, and proof-backed reuse rejection
- open [Compliance-Gated Bulletin Workflow](../operations/guides/compliance-gated-bulletin-workflow.md)
  when you want a real execution path for retry evidence, source-run input
  rematerialization, and repair verification
- open [Branching Bulletin Workflow](../operations/guides/branching-bulletin-workflow.md)
  when you want a real execution path for branch decisions, skipped-lane
  evidence, and replay stability

## Code Anchors

- `crates/bijux-dag-runtime/README.md`
- `crates/bijux-dag-runtime/CONTRACT.md`
- `crates/bijux-dag-runtime/src/lib.rs`
- `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs`
- `crates/bijux-dag-runtime/src/policy/evaluator.rs`
- `crates/bijux-dag-runtime/src/replay/verifier.rs`

## Review Focus

- runtime policy should stay explicit, testable, and separate from graph definition
- artifact and replay rules should be inspectable rather than hidden behind execution helpers
- cache, replay, and rerun reuse rules should stay auditable at crate boundaries
- package ownership should remain focused on execution-time behavior only
