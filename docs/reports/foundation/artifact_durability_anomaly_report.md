# Artifact Durability Anomaly Report

## Scope

Durability anomalies include partial writes, checksum mismatches, manifest/index inconsistency, corruption fixtures, and unsafe rebuild behavior.

## Detection anchors

- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs`
- `crates/bijux-dag-artifacts/tests/artifact_identity_lifecycle_completion_contracts.rs`

## Governance references

- `configs/suites/artifact_durability_verification.json`
- `evidence/cache/artifact_durability/regression_corpus.json`
