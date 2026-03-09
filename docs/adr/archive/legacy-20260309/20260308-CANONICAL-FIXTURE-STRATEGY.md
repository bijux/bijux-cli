# ADR: Canonical Fixture Strategy

## Status

Accepted

## Context

Fixture volume and overlap increase maintenance cost and reduce signal clarity in tests and governance checks.

## Decision

1. Govern fixture families with explicit owner/suite metadata and family roots.
2. Standardize fixture tags: `canonical`, `stress`, `corrupt`, `smoke`, `legacy`.
3. Restrict smoke defaults to canonical/smoke tagged fixtures.
4. Track unconsumed and duplicate fixtures as contraction priorities.

## Consequences

- Fixture discovery and ownership become explicit.
- Duplicate and orphan fixture cleanup becomes auditable.
- Default smoke workflows remain stable and low-noise.

## Enforcement

- `configs/policy/fixture_family_governance.json`
- `configs/suites/fixture_contraction_verification.json`
- `crates/bijux-dev-dag/tests/fixture_canonicalization_contracts.rs`
