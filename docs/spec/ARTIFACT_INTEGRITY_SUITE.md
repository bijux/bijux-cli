# Artifact integrity suite

## Scope

Defines the artifact integrity verification suite for run-directory storage, manifest validation, corruption detection, and replay/import-export artifact checks.

## Primary verification surfaces

- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dag-artifacts/tests/conformance.rs`
- `crates/bijux-dag-artifacts/tests/fixtures/corrupt_runs/*`

## Control-plane enforcement

- suite id: `artifact-hardening`
- command: `bijux-dev-dag run-dir-audit --run-dir <path> [--strict]`

## Coverage intent

- manifest validation
- run import/export payload validation
- corruption fixture rejection
- replay artifact payload verification
- retention-aware cleanup planning
