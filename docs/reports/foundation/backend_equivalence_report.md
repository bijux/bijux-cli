# Backend Equivalence Report

Generated: 2026-03-08

## Equivalence scope

- local, kubernetes, hpc, and remote backend equivalence fixtures
- replay compatibility across backend-originated runs
- portability proof and downgrade classification surfaces

## Evidence and test anchors

- fixture corpus:
  - `evidence/compat/backend_equivalence/generated_fixture_corpus.json`
- completion contracts:
  - `crates/bijux-dev-dag/tests/backend_equivalence_contracts.rs`
  - `crates/bijux-dev-dag/tests/backend_equivalence_completion_contracts.rs`
  - `crates/bijux-dev-dag/tests/replay_equivalence_completion_contracts.rs`
- CLI and app surfaces:
  - `crates/bijux-dag-cli/tests/contract_surface.rs`
  - `crates/bijux-dag-app/src/routes/surface_routes.rs`

## Current posture

- cross-backend semantic equivalence coverage is broad across compatibility fixtures
- replay and portability equivalence behavior remains contract-governed
