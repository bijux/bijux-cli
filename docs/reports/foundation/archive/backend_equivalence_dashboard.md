# Backend Equivalence Dashboard

Generated: 2026-03-08

## Coverage signals

- backend equivalence contracts:
  - `crates/bijux-dev-dag/tests/backend_equivalence_contracts.rs`
  - `crates/bijux-dev-dag/tests/backend_equivalence_completion_contracts.rs`
- replay compatibility contracts:
  - `crates/bijux-dev-dag/tests/replay_equivalence_completion_contracts.rs`
- CLI/operator surfaces:
  - `crates/bijux-dag-cli/tests/contract_surface.rs`
  - `crates/bijux-dag-app/src/routes/surface_routes.rs`

## Reported signals

- `docs/reports/foundation/backend_equivalence_report.md`
- `docs/reports/foundation/backend_capability_matrix.md`
- `docs/reports/foundation/backend_divergence_diagnostics_report.md`
- `docs/reports/foundation/backend_equivalence_quality_benchmark.md`
- `docs/reports/foundation/backend_equivalence_performance_benchmarks.md`

## Current status

- backend semantic equivalence coverage: present
- replay and portability compatibility coverage: present
- divergence diagnostics and capability matrix reporting: present
