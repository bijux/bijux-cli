# bijux-dag-runtime

`bijux-dag-runtime` is the execution engine for `bijux-dag`. It owns runtime
planning, scheduling, adapter invocation boundaries, policy checks, replay
classification, cache behavior, and trace emission.

## What this crate provides

- Execution planning and node orchestration.
- Policy evaluation and runtime diagnostics.
- Replay, diff, cache, and artifact integration behavior.
- Adapter boundaries for local and external execution backends.

Choose this crate when you need to execute validated DAG graphs or integrate
with Bijux runtime policies from Rust.

## Deliberate boundaries

This crate does not own:

- authoritative graph schema and validation rules,
- top-level command parsing or output presentation,
- release-governance and maintainer report composition.

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

## Public Rust surface

- prefer focused crate-root imports for a small number of runtime types or functions
- use `bijux_dag_runtime::stable` for the explicit long-lived compatibility lane
- use `bijux_dag_runtime::prelude` for common planning and execution workflows
- use `bijux_dag_runtime::simulated_platform` only for deliberate modeled-platform
  and control-plane evidence work
- treat backend-heavy compatibility helpers as repository-owned support surface,
  not as the primary docs-facing import lane

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/)
