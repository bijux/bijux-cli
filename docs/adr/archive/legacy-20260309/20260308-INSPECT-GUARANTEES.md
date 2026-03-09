# ADR: Inspect Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

Inspect is a primary operator-read surface for run diagnostics, timeline state,
lineage interpretation, and corruption triage.

## Decision

Inspect guarantees are:

1. Inspect outputs remain deterministic, schema-governed, and snapshot-stable.
2. Inspect behavior remains robust under corrupted, partial, imported, and replayed runs.
3. Inspect routes and malformed-input entrypoints remain no-panic.
4. Inspect diagnostics and lineage visibility remain represented in generated reports.

## Enforcement

- Status mapping:
  - `docs/reports/foundation/INSPECT_321_340_STATUS_REPORT.md`
- Diagnostics and stability reports:
  - `docs/reports/foundation/INSPECT_DIAGNOSTICS_REPORT.md`
  - `docs/reports/foundation/INSPECT_STABILITY_REPORT.md`
- Diagnostics dashboard:
  - `docs/reports/foundation/INSPECT_DIAGNOSTICS_DASHBOARD.md`
- Verification suite:
  - `configs/suites/inspect_verification.json`

## Consequences

- Inspect semantics are treated as a governed product contract.
- Inspect behavior changes must preserve determinism and diagnostics guarantees.
