# Artifact Integrity Fixture Inventory Report

Generated: 2026-03-08

Manifest fixtures:
- `crates/bijux-dag-artifacts/tests/fixtures/run_manifest_minimal.json`
- `crates/bijux-dag-artifacts/tests/fixtures/run_manifest_maximal.json`

Integrity and retention fixture-linked tests:
- `run_manifest_roundtrip_and_retention_contracts.rs`
- `run_manifest_identity_contracts.rs`
- `artifact_hardening_contracts.rs`

IO/store/hardening direct fixture behaviors:
- Empty and non-empty payload indexing in nested paths.
- Lineage-only preservation explain paths.
- Missing payload and damaged manifest metadata anomaly classification.
- Retained-root vs collectable-leaf gc explain stability.

Inventory note:
- This report intentionally tracks active fixture-bearing artifact tests only; speculative fixture paths are excluded.
