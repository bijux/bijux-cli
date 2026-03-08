# Full-Lane Only Tests Report

generated_from: `make test-all` inventory comparison against `make test`

These suites are intentionally full-lane only because they are either long-running
or require complete evidence/governance validation:

- `crates/bijux-dev-dag/tests/benchmark_governance_contracts.rs`
- `crates/bijux-dev-dag/tests/release_evidence_linkage_contracts.rs`
- `crates/bijux-dag-runtime/tests/performance_capacity_contracts.rs`

The machine-readable source of truth is:

- `docs/reports/foundation/full_lane_unique_inventory.json`
