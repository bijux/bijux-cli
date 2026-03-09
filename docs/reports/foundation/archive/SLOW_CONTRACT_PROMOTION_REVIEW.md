# Slow Contract Promotion Review

generated_from: `test duration telemetry + release-critical classification`

Reviewed slow contracts for potential promotion into `make test`:

1. `bijux-dag-cli::contract_surface::semantic_portability_backend_query_surface_is_available`
result: keep in slow lane (`test-slow` and `test-all`) because duration is unstable in local developer environments.

2. `bijux-dev-dag::evidence_governance_contract::evidence_governance_contract_enforces_ownership_and_freeze`
result: keep in slow lane because it validates broad evidence topology and ownership checks.

3. `crates/bijux-dag-runtime/tests/performance_capacity_contracts.rs`
result: keep in full lane because it is a benchmark-regression governance gate.

Conclusion:
- no currently slow contracts are promoted into `make test` in this revision.
- revisit promotion after fixture/runtime simplification reduces variance.
