# bijux-dag-runtime

`bijux-dag-runtime` is the execution engine for `bijux-dag`. It owns runtime
planning, scheduling, adapter invocation boundaries, policy checks, replay
classification, cache behavior, and trace emission.

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

Choose this crate when you need to execute validated DAG graphs or integrate
with Bijux runtime policies from Rust.

## What It Does Not Own

- authoritative graph schema and validation rules
- top-level command parsing or output presentation
- release-governance and maintainer report composition

## Runtime identity rules

- runtime manifests and provenance records stamp the crate package version
  directly from build metadata
- an optional Git short SHA may be appended at build time when the crate is
  compiled from a repository checkout or injected through
  `BIJUX_DAG_BUILD_GIT_SHA` for release-tree builds
- runtime execution does not shell out to `git` to discover version identity
- replay and cache identity therefore do not depend on the operator's current
  working directory or any unrelated Git repository around the binary

Use these rules when reviewing runtime fingerprint drift or provenance output.

## Public Rust Surface

- prefer focused crate-root imports for a small number of runtime types or functions
- use `bijux_dag_runtime::stable` for the explicit long-lived compatibility lane
- use `bijux_dag_runtime::prelude` for common planning and execution workflows
- use `bijux_dag_runtime::simulated_platform` only for deliberate modeled-platform
  and control-plane evidence work
- treat backend-heavy compatibility helpers as repository-owned support surface,
  not as the primary docs-facing import lane

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

For the repository-backed example that exercises branch decisions, join trigger
evaluation, skipped-lane evidence, and replay stability, use
[Branching Bulletin Workflow](../../docs/bijux-dag/operations/guides/branching-bulletin-workflow.md).

For the repository-backed example that exercises retry accounting, replay
boundary input rematerialization, and post-repair verification on a failed run,
use
[Compliance-Gated Bulletin Workflow](../../docs/bijux-dag/operations/guides/compliance-gated-bulletin-workflow.md).

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/)
