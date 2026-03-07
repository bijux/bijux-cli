# Evidence Consumption by Crate

## Purpose

This report summarizes which evidence families are consumed by each crate test surface.

## bijux-dag-core

- `crates/bijux-dag-core/tests/examples_contract.rs`
  - consumes `evidence/authoring/examples/**`
- `crates/bijux-dag-core/tests/compat_contract.rs`
  - consumes `evidence/compat/**`

## bijux-dag-runtime

- `crates/bijux-dag-runtime/tests/infrastructure_fixture_contract.rs`
  - consumes `evidence/perf/fixtures/infrastructure/**`
- runtime battle and fault contracts
  - consume `evidence/battle/**` and `evidence/fault/**`

## bijux-dag-app

- `crates/bijux-dag-app/tests/replay_contract.rs`
  - consumes `evidence/cache/replay/**`
- `crates/bijux-dag-app/tests/comparison_harness_contract.rs`
  - consumes `evidence/compare/**`

## bijux-dag-artifacts

- artifact compatibility and import/export contracts
  - consume `evidence/compat/**` and import-export evidence families

## bijux-dev-dag

- control-plane evidence contracts
  - consume `evidence/_meta/registries/**`
  - consume `evidence/reports/**`
  - validate ownership, trust properties, and consumer drift

## Policy

- Canonical scenario roots must stay under `evidence/`.
- Crate consumers are read-only and must not mutate evidence assets.
