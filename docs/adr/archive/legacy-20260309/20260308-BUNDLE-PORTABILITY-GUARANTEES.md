# ADR: Bundle Portability Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

Bundle import/export is a core portability boundary across operators, environments,
and replay workflows. Portability correctness requires stable format behavior,
explicit failure diagnostics, and compatibility governance.

## Decision

Bundle portability guarantees are:

1. Export/import behavior remains reproducible and verify-only safe.
2. Corrupted, truncated, or unsupported bundles are rejected with clear diagnostics.
3. Imported bundles remain replay-compatible and provenance-preserving.
4. Schema drift visibility and fsck verification remain enforced in governance suites.

## Enforcement

- Status mapping:
  - `docs/reports/foundation/BUNDLE_PORTABILITY_261_280_STATUS_REPORT.md`
- Portability and diagnostics reports:
  - `docs/reports/foundation/BUNDLE_PORTABILITY_REPORT.md`
  - `docs/reports/foundation/BUNDLE_IMPORT_DIAGNOSTICS_REPORT.md`
- Compatibility dashboard:
  - `docs/reports/foundation/BUNDLE_COMPATIBILITY_DASHBOARD.md`
- Verification suite:
  - `configs/suites/bundle_portability_verification.json`

## Consequences

- Bundle portability becomes an explicit correctness and operator-trust contract.
- Changes to bundle format or import behavior must preserve governed compatibility signals.
