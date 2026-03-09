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
