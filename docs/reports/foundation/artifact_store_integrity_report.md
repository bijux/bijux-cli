# Artifact Store Integrity Report

Generated: 2026-03-08

## Integrity surfaces

- atomic durable writes:
  - `crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- checksum and corruption handling:
  - `crates/bijux-dag-artifacts/tests/artifact_identity_lifecycle_completion_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_io_expansion_contracts.rs`
- run-dir verification and anomaly reporting:
  - `crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_identity_lifecycle_completion_contracts.rs`

## Current posture

- store write/read integrity is covered with corruption and recovery checks
- run-directory verification rejects incomplete or inconsistent artifact state
- checksum surfaces are exercised via regression corpora and lifecycle contracts
