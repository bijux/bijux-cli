# Runtime Allowed Dependencies

This document defines runtime crate dependency boundaries.

## Runtime crate (`bijux-dag-runtime`) allowed direct dependencies

- workspace crates: `bijux-dag-core`, `bijux-dag-artifacts`
- runtime support crates: `serde`, `serde_json`, `sha2`, `hex`, `thiserror`, `ctrlc`

## Runtime disallowed dependency classes

- CLI and command-routing crates (`clap`, `bijux-dag-cli`, `bijux-dag-app`)
- network/server orchestration crates (`axum`, `warp`, `octocrab`, `reqwest`, `git2`)

## Enforcement

- `crates/bijux-dev-dag/tests/no_cli_in_runtime.rs`
- `crates/bijux-dev-dag/tests/runtime_contraction_contracts.rs`
- `crates/bijux-dev-dag/tests/dependency_boundary_contracts.rs`
