# Artifact Store Truth Report

Generated: 2026-03-08

Scope:
- `crates/bijux-dag-artifacts/src/io/fs.rs`
- `crates/bijux-dag-artifacts/src/io/store.rs`
- `crates/bijux-dag-artifacts/src/storage/services.rs`
- `crates/bijux-dag-artifacts/src/storage/hardening.rs`

Direct evidence from tests:
- `artifact_io_store_hardening_expansion_contracts.rs`
- `artifact_io_expansion_contracts.rs`
- `artifact_storage_resilience_contracts.rs`
- `io_store_fs_contracts.rs`
- `storage_services_contracts.rs`

Pinned truth points:
- Normalized relative fs paths are accepted and escaping paths are rejected.
- Filesystem artifact-store writes are repeatable and latest-write wins.
- Modeled object-store capability surface remains read/write disabled and explicit.
- Run-dir verification continues to surface missing payload/index/trace anomalies.
- Cleanup planning and lineage explain keep preserved vs collectable decisions stable.
