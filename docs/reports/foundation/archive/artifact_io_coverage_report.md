# Artifact IO Coverage Report

Generated from contract tests and fixture-backed artifact workflows.

## Source coverage map

- `crates/bijux-dag-artifacts/src/io/fs.rs`
  - `crates/bijux-dag-artifacts/tests/io_store_fs_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_io_expansion_contracts.rs`
- `crates/bijux-dag-artifacts/src/io/store.rs`
  - `crates/bijux-dag-artifacts/tests/io_store_fs_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_io_expansion_contracts.rs`
- `crates/bijux-dag-artifacts/src/storage/services.rs`
  - `crates/bijux-dag-artifacts/tests/storage_services_contracts.rs`
- `crates/bijux-dag-artifacts/src/storage/hardening.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs`

## Behavior coverage map

- fs path safety: traversal rejection + normalized relative path checks.
- roundtrip reliability: nested trees, empty files, empty directories, binary payloads.
- large payload handling: chunk-equivalent hashing on multi-MB buffers.
- provenance behavior: duplicate-content distinct-provenance identity expectations.
- inspect resilience: missing payload and corrupted index handling via app-level inspect path.
- retention and gc explain: stable preserve/collect decisions and cleanup plan invariants.
- store capability stability: implemented vs modeled-only capability declarations.

## Fast suite linkage

- `configs/suites/artifact_io_zero_coverage_fast.json`
- `crates/bijux-dev-dag/tests/artifact_io_fast_suite_contracts.rs`
