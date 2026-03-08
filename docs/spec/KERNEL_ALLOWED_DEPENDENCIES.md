# Kernel Allowed Dependencies

This document defines dependency rules for the deterministic kernel path.

## Scope

Kernel scope is defined by:

- `docs/spec/KERNEL_BOUNDARY_CONTRACT.md`
- `configs/policy/kernel_dependency_policy.json`

## Core crate (`bijux-dag-core`) allowed dependencies

- `serde`
- `serde_json`
- `sha2`
- `hex`
- `thiserror`
- `criterion` (dev-only benchmark dependency)
- `tempfile` (dev-only test dependency)

## Runtime kernel path disallowed dependency classes

- CLI parsing libraries (`clap`)
- HTTP/server frameworks (`axum`, `warp`)
- repository/network governance clients (`git2`, `octocrab`, `reqwest`)
- app/dev crates (`bijux-dag-app`, `bijux-dev-dag`)

## Enforcement

- `crates/bijux-dev-dag/tests/no_runtime_in_core.rs`
- `crates/bijux-dev-dag/tests/runtime_contraction_contracts.rs`
- `crates/bijux-dev-dag/tests/dependency_boundary_contracts.rs`
