# Cache Integrity Coverage Report

## Coverage matrix

| Coverage class | Anchor |
| --- | --- |
| key determinism and lookup consistency | `docs/spec/CACHE_CONTRACT.md` |
| invalidation on graph/environment/artifact/replay changes | `docs/spec/CACHE_CONTRACT.md`, `docs/spec/REPLAY_CONTRACT.md` |
| corruption detection and integrity verification | `evidence/cache/metadata.json`, `evidence/cache/corrupt/hash_mismatch.json` |
| concurrency, eviction, retention, lifecycle | `docs/spec/CONCURRENCY_MODEL.md`, `docs/spec/CACHE_PRUNE_POLICY.md`, `docs/spec/ARTIFACT_RETENTION_POLICY.md` |
| stress, performance, telemetry | `evidence/cache/scenarios/warm_cold.json`, `docs/reports/foundation/performance_evidence_report.md`, `docs/spec/RUNTIME_TELEMETRY_SCHEMA.md` |
| explainability integration | `docs/spec/EXPLAIN_SURFACES_CONTRACT.md` |

## Completion signals

- contract: `docs/spec/CACHE_SYSTEM_INTEGRITY_CONTRACT.md`
- suite: `configs/suites/cache_integrity_verification.json`
- corpus: `evidence/cache/cache_integrity/regression_corpus.json`
