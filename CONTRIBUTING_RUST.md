# Contributing to the Rust Foundation

## Purpose
This guide defines engineering standards for the Rust workspace in `bijux-cli`.

## Workspace layout
- `crates/bijux-cli-contracts`: shared durable contracts.
- `crates/bijux-cli-core`: execution kernel primitives.
- `crates/bijux-cli-routing`: command graph and resolution.
- `crates/bijux-cli-output`: output encoders and envelopes.
- `crates/bijux-cli-repl`: interactive shell orchestration.
- `crates/bijux-cli-plugin`: plugin lifecycle boundaries.
- `crates/bijux-cli-python`: Python compatibility bridge.
- `crates/bijux-cli-install`: install/update flow boundaries.
- `crates/bijux-cli-bin`: binary entrypoint.

## Non-negotiable rules
- `unsafe` is forbidden workspace-wide.
- Crate dependency boundaries must pass `architecture_boundaries` tests.
- New public contract types belong in `bijux-cli-contracts`.
- Command behavior changes must preserve documented compatibility contracts.

## Local validation commands
- `cargo fmt --all`
- `cargo fmt-check`
- `cargo check-workspace`
- `cargo lint`
- `cargo test --workspace`
- `cargo test -p bijux-cli-core --test architecture_boundaries`

## Dependency policy
- Keep dependencies minimal and justified.
- Use crates from `crates.io` only unless a security exception is documented.
- Run policy checks with `cargo deny check` when `cargo-deny` is installed.

## Design review checklist
- Does the change preserve root grammar and namespace contracts?
- Does the change preserve exit-code compatibility?
- Does the change preserve stdout/stderr routing rules?
- Does the change preserve plugin namespace and lifecycle contracts?
- Does the change include tests for new behavior?

## Commit conventions
- Use Conventional Commits.
- Keep commits small and logically grouped.
- Prefer one behavior change per commit.

## Maintainer milestone reporting
- Every milestone report must include explicit `done`, `left`, and `blocked/deferred` sections.
- Milestone completion claims require evidence artifacts:
  - `artifacts/status/what_is_done.json`
  - `artifacts/status/what_is_left.json`
  - `artifacts/status/what_is_partial.json`
  - `artifacts/status/what_is_deferred.json`
