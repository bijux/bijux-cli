# Backend Divergence Diagnostics Report

Generated: 2026-03-08

## Divergence diagnostic surfaces

- equivalence proof downgrade reporting:
  - `crates/bijux-dag-app/src/routes/surface_routes.rs`
  - `crates/bijux-dag-cli/tests/contract_surface.rs`
- replay mismatch and drift diagnostics:
  - `crates/bijux-dev-dag/tests/replay_equivalence_completion_contracts.rs`
  - `docs/reports/foundation/replay_equivalence_diagnostics_report.md`
- backend quality signals:
  - `docs/reports/foundation/backend_equivalence_quality_benchmark.md`

## Current posture

- backend divergence is surfaced through explicit downgrade and mismatch diagnostics
- divergence evidence is tied to generated compatibility fixtures and contract suites
