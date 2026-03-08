# Artifact Durability Coverage Report

## Coverage matrix

| Coverage class | Anchor |
| --- | --- |
| atomic write and read consistency | `crates/bijux-dag-artifacts/src/storage/hardening.rs` |
| partial-write and corruption recovery | `docs/spec/ARTIFACT_STORAGE_LIFECYCLE_CONTRACT.md` |
| concurrent-write and GC race safety | `docs/spec/CONCURRENCY_MODEL.md` |
| checksum verification and corruption detection | `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs` |
| rebuild, compaction, fragmentation | `docs/reports/foundation/artifact_storage_lifecycle_benchmarks.md` |
| retention durability and lifecycle recovery | `docs/spec/ARTIFACT_RETENTION_POLICY.md` |
| telemetry and anomaly detection | `docs/reports/foundation/artifact_storage_lifecycle_telemetry_report.md` |

## Completion signals

- contract: `docs/spec/ARTIFACT_DURABILITY_GUARANTEES_CONTRACT.md`
- suite: `configs/suites/artifact_durability_verification.json`
- corpus: `evidence/cache/artifact_durability/regression_corpus.json`
