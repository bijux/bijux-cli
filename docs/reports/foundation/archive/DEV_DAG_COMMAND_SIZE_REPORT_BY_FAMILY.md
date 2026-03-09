# Dev-Dag Command Size Report by Family

Generated on 2026-03-08 using `wc -l` snapshots.

| Family | File | Lines |
| --- | --- | ---: |
| authoring | `commands/authoring_evidence.rs` | 347 |
| battle | `commands/battle_evidence.rs` | 444 |
| benchmark | `commands/benchmark_harness.rs` | 135 |
| compare | `commands/compare_evidence.rs` | 229 |
| access | `commands/evidence_access.rs` | 456 |
| control-plane | `commands/evidence_control_plane.rs` | 547 |
| registry | `commands/evidence_registry.rs` | 333 |
| model | `commands/model.rs` | 68 |
| performance | `commands/perf_evidence.rs` | 268 |
| catalog | `commands/suite_catalog.rs` | 321 |
| catalog-data | `commands/suite_catalog_repo.inc` | 672 |
| verification-binary | `src/bin/attestation_verify.rs` | 109 |
| verification-binary | `src/bin/integrated_verify.rs` | 113 |
| verification-binary | `src/bin/migration_simulate.rs` | 102 |
| verification-binary | `src/bin/trust_health.rs` | 52 |
| command-router | `commands/mod.rs` | 8781 |

## Summary

`commands/mod.rs` remains the largest command surface and has been reduced by extracting file-catalog helpers into `commands/file_catalog.rs`.
