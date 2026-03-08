# Dev Governance Allowed Dependencies

This document defines dependencies for governance tooling in `bijux-dev-dag`.

## `bijux-dev-dag` allowed direct dependency classes

- workspace crates used for contracts and reports:
  - `bijux-dag-core`
  - `bijux-dag-runtime`
  - `bijux-dag-artifacts`
- governance/tooling crates:
  - `clap`
  - `serde`
  - `serde_json`
  - `sha2`
  - `hex`
  - `tempfile`

## Disallowed direct dependencies

- app routing crate: `bijux-dag-app`

## Enforcement

- `crates/bijux-dev-dag/tests/crate_taxonomy_guardrails.rs`
- `crates/bijux-dev-dag/tests/dependency_boundary_contracts.rs`
