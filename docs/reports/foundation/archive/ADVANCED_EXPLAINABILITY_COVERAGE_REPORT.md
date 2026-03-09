# Advanced Explainability Coverage Report

## Coverage matrix

| Coverage class | Anchor |
| --- | --- |
| node-level execution explanation | `crates/bijux-dag-app/tests/diff_explain_contract.rs` |
| scheduler and replay decision explanation | `docs/spec/SCHEDULER_CONTRACT.md`, `docs/spec/REPLAY_CONTRACT.md` |
| cache hit/miss explanation | `docs/spec/CACHE_CONTRACT.md` |
| lineage and dependency explanation | `crates/bijux-dag-app/tests/artifact_identity_explain_contract.rs` |
| environment/backend mismatch explanation | `crates/bijux-dev-dag/tests/environment_identity_completion_contracts.rs`, `crates/bijux-dev-dag/tests/backend_equivalence_completion_contracts.rs` |
| schema and snapshot stability | `configs/schema/operator/run_explain_failure.schema.json`, `crates/bijux-dag-app/tests/snapshots/route_detailed_wording.txt` |
| stress and performance | `configs/suites/explain_surface_stress.json`, `docs/reports/foundation/EXPLAINABILITY_BENCHMARKS.md` |

## Completion signals

- contract: `docs/spec/ADVANCED_EXPLAINABILITY_MODEL_CONTRACT.md`
- suite: `configs/suites/advanced_explainability_regression.json`
- corpus: `evidence/cache/explainability/regression_corpus.json`
