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

Use this page when the question is about what happens after a graph has already
been accepted as valid.

The intended Rust import lanes are the crate root, `stable`, and `prelude`.
At the module level, the public lanes are intentionally limited to `stable`,
`prelude`, `experimental`, and `simulated_platform`. Hidden compatibility
helpers remain available for repository-owned coverage, and the `experimental`
lane is opt-in behind `experimental-public-api`.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| execution engine | planning, scheduler behavior, backend invocation, replay semantics |
| runtime policy | policy evaluation, trace emission, error classification, capability checks |
| local container execution | engine detection, mounted input and output layout, stdout/stderr capture, and recorded image identity |
| branch execution | selected-lane pruning, skipped-node recording, trigger-rule evaluation, and replay equivalence checks |
| runtime artifacts | manifests, verification, cache lineage, and proof material |
| runtime identity | build-stamped version identity and deterministic runtime fingerprints |
| boundary | does not own authoritative DAG schema or user-facing CLI routing |

## Identity Guarantees

- runtime manifests and provenance records derive `tool_version` from the crate
  build, not from the operator's current shell environment
- a Git short SHA may appear only when it was captured during the build itself
  or injected through `BIJUX_DAG_BUILD_GIT_SHA` for a clean release tree
- runtime fingerprints stay stable when the same binary is executed from a
  different working directory
- unrelated ambient Git repositories are not allowed to rewrite replay or cache
  identity

## Source Layout

- `crates/bijux-dag-runtime/src/runtime_core`
- `crates/bijux-dag-runtime/src/adapters`
- `crates/bijux-dag-runtime/src/backend`
- `crates/bijux-dag-runtime/src/artifacts`
- `crates/bijux-dag-runtime/src/cache`
- `crates/bijux-dag-runtime/src/policy`
- `crates/bijux-dag-runtime/src/replay`
- `crates/bijux-dag-runtime/src/diagnostics`

## Open Next

- open the [DAG Handbook](../index.md) for the full DAG system map
- open [`bijux-dag-core`](./bijux-dag-core.md) for graph truth and planning inputs
- open [`bijux-dag-app`](./bijux-dag-app.md) for command orchestration and response shaping
- open [Container Packaging Workflow](../operations/guides/container-packaging-workflow.md) for the repository-backed execution path that exercises retained container identity and engine-unavailable failure handling
- open [Branching Bulletin Workflow](../operations/guides/branching-bulletin-workflow.md) for the repository-backed execution path that exercises branch decisions, join-trigger behavior, and replay stability
- open [Compliance-Gated Bulletin Workflow](../operations/guides/compliance-gated-bulletin-workflow.md) for the repository-backed execution path that exercises retry evidence, source-run input rematerialization, and strict verification after repair
- open [Scheduled Catalog Refresh Workflow](../operations/guides/scheduled-catalog-refresh-workflow.md) for the repository-backed execution path that exercises cron preview, deterministic schedule run ids, queue dispatch, and explicit ledger-to-run completion

## Code Anchors

- `crates/bijux-dag-runtime/README.md`
- `crates/bijux-dag-runtime/CONTRACT.md`
- `crates/bijux-dag-runtime/src/lib.rs`
- `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs`
- `crates/bijux-dag-runtime/src/policy/evaluator.rs`
- `crates/bijux-dag-runtime/src/replay/verifier.rs`

## Review Lens

- runtime policy should stay explicit, testable, and separate from graph definition
- artifact and replay rules should be inspectable rather than hidden behind execution helpers
- cache, replay, and rerun reuse rules should stay auditable at crate boundaries
- package ownership should remain focused on execution-time behavior only
