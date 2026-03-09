# Inspect and Diagnostics Guarantees

Status: accepted
Owner: operator surface maintainers
Date: 2026-03-09

## Decision
Inspect and diagnostics outputs remain operator-focused, deterministic, and resilient to malformed inputs.

## Consequences
- Diagnostics routes prioritize no-panic behavior.
- Human and machine output contracts remain explicit.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-INSPECT-GUARANTEES.md
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

### SOURCE: 20260308-EXPLAIN-SEMANTICS-GUARANTEES.md
# ADR: Explain Semantics Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

Explain surfaces are operator-facing trust mechanisms for replay, drift, cache,
lineage, and failure interpretation. They must be stable, deterministic, and
diagnostic under degraded state.

## Decision

Explain semantics guarantees are:

1. Explain outputs remain deterministic and schema-governed.
2. Explain behavior remains stable across partial, corrupted, imported, and replayed runs.
3. Explain reasoning surfaces for replay, lineage, scheduler, and plan stay operator-visible.
4. Explain regression and anomaly signals remain enforced in verification suites.

## Enforcement

- Status mapping:
  - `docs/reports/foundation/EXPLAIN_301_320_STATUS_REPORT.md`
- Coverage/determinism reports:
  - `docs/reports/foundation/EXPLAIN_COVERAGE_REPORT.md`
  - `docs/reports/foundation/EXPLAIN_DETERMINISM_REPORT.md`
- Diagnostics dashboard:
  - `docs/reports/foundation/EXPLAIN_DIAGNOSTICS_DASHBOARD.md`
- Verification suite:
  - `configs/suites/explain_verification.json`

## Consequences

- Explain semantics become a governed product contract.
- Explain-surface changes must preserve schema, snapshot, and determinism guarantees.
