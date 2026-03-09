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
  - `docs/reports/foundation/explain_301_320_status_report.md`
- Coverage/determinism reports:
  - `docs/reports/foundation/explain_coverage_report.md`
  - `docs/reports/foundation/explain_determinism_report.md`
- Diagnostics dashboard:
  - `docs/reports/foundation/explain_diagnostics_dashboard.md`
- Verification suite:
  - `configs/suites/explain_verification.json`

## Consequences

- Explain semantics become a governed product contract.
- Explain-surface changes must preserve schema, snapshot, and determinism guarantees.
