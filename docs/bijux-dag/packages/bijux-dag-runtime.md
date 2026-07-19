---
title: bijux-dag-runtime Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# bijux-dag-runtime

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-runtime?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-runtime)
[![Rust docs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-dag-runtime docs](https://img.shields.io/badge/docs-runtime-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-runtime` owns execution-time behavior for DAG runs: planning,
scheduler policy, adapter invocation boundaries, artifact writing, replay,
cache policy, and runtime diagnostics.

At the product level, `bijux-dag` v0.4.0 is a local-first DAG runtime for
reproducible workflows with explicit graph contracts, deterministic execution
records, verified artifacts, cache explanation, and replayable run bundles.
The [Replay Contract](../../spec/REPLAY_CONTRACT.md) defines the replay authority.
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

## Contract Authorities

| Question | Authority |
| --- | --- |
| graph, execution, environment, cache, and replay identity | [Reproducibility Model](../interfaces/reproducibility-model.md) |
| retained lifecycle and node evidence | [Run Evidence Layout](../interfaces/run-evidence-layout.md) |
| adapter handshake and execution protocol | [Adapter Contract](../../spec/ADAPTER_CONTRACT.md) |
| security and subprocess isolation limits | [Execution Security And Isolation](../operations/security-isolation-truth.md) |
| Rust imports and implementation modules | [crate README](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-runtime) and [docs.rs](https://docs.rs/bijux-dag-runtime) |

## Practical Starting Points

- open the [DAG Handbook](../index.md) for the full DAG system map
- open [`bijux-dag-core`](bijux-dag-core.md) for graph truth and planning
  inputs
- open [`bijux-dag-app`](bijux-dag-app.md) for command orchestration and
  response shaping
- open [Cache Behavior Workflow](../operations/cache-behavior-workflow.md)
  when you want a real execution path for cache hits, invalidation, corruption
  refusal, and proof-backed reuse rejection
- open [Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md)
  when you want a real execution path for retry evidence, source-run input
  rematerialization, and repair verification
- open [Branching Bulletin Workflow](../operations/branching-bulletin-workflow.md)
  when you want a real execution path for branch decisions, skipped-lane
  evidence, and replay stability

## Review Focus

- runtime policy should stay explicit, testable, and separate from graph definition
- artifact and replay rules should be inspectable rather than hidden behind execution helpers
- cache, replay, and rerun reuse rules should stay auditable at crate boundaries
- package ownership should remain focused on execution-time behavior only
