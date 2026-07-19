# Test Trust Maintenance Report

## Purpose

This report records the surfaces that keep runtime test classification and
maintenance decisions reviewable.

## Guarded surfaces

- policy ledger: `configs/dag/policy/test_trust_ledger.json`
- ledger contract: `docs/spec/TEST_TRUST_LEDGER.md`
- maintenance tests: `crates/bijux-dev/tests/test_trust_maintenance_contracts.rs`
- runtime catalog: `crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json`

## Current maintenance stance

- every runtime test file should land in an explicit trust class
- must-never-break surfaces must stay out of cosmetic and duplicate lanes
- snapshot assertions stay restricted to the allowlist in the ledger policy
