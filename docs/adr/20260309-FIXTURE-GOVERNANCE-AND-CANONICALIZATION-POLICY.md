# Fixture Governance and Canonicalization Policy

Status: accepted
Owner: test governance maintainers
Date: 2026-03-09

## Decision
Fixture corpora are canonicalized, owned, and lifecycle-governed to support deterministic contracts and repeatable diagnostics.

## Consequences
- Fixtures are treated as controlled evidence assets.
- Fixture drift is governed by explicit policy.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-CANONICAL-FIXTURE-STRATEGY.md
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

### SOURCE: 20260308-FIXTURE-OWNERSHIP-AND-LIFECYCLE-GOVERNANCE.md
# ADR: Fixture Ownership and Lifecycle Governance

- Date: 2026-03-08
- Status: Accepted

## Context

Fixture usage expanded across core, app, runtime, artifacts, benchmark, and evidence paths without a single ownership map. That causes duplicated fixture semantics, orphan fixture files, and unclear review responsibility when fixture changes affect release gates.

## Decision

Adopt a governed fixture family policy at `configs/policy/fixture_family_governance.json` with mandatory fields per family:

- `fixture_purpose`
- `fixture_owner`
- `fixture_lane`
- `fixture_taxonomy`
- `owner_suite`
- `owner_crate`

Generate deterministic governance reports from policy and filesystem state:

- Family inventories with owner suite and crate
- Missing ownership reports
- Unreferenced fixture report
- Duplicate semantic-hash fixture report
- Stale fixture schema-field report

## Consequences

Positive:

- Every governed fixture family has explicit purpose and ownership.
- Fixture review responsibility is clear during release hardening.
- Drift and duplication are visible via generated reports.

Tradeoff:

- Policy and generated reports must be refreshed as families evolve.

## Follow-up

- Keep generator output under `docs/reports/foundation` current in governance updates.
- Extend suite-level tests to enforce policy freshness if drift appears repeatedly.
