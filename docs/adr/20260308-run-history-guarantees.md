# ADR: Run History Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

Run history is a primary operator-read surface. It must remain deterministic,
recoverable under damaged state, and stable across schema and workspace changes.

## Decision

Run history guarantees are:

1. History ordering and pagination behavior remain deterministic.
2. History reconstruction from raw run directories is resilient to corruption.
3. Identity and inspect surfaces remain schema-lockstep stable.
4. Relocation, alias updates, and partial artifact deletion do not break history.
5. Diagnostics remain available for damaged-run and strict verification paths.

## Enforcement

- Status mapping:
  - `docs/reports/foundation/run_history_221_240_status_report.md`
- Size and resilience reports:
  - `docs/reports/foundation/run_history_size_growth_report.md`
  - `docs/reports/foundation/run_history_corruption_resilience_report.md`
- Diagnostics and consistency:
  - `docs/reports/foundation/run_history_diagnostics_report.md`
  - `docs/reports/foundation/run_history_consistency_dashboard.md`
- Verification suite:
  - `configs/suites/run_history_invariants.json`

## Consequences

- Run history becomes a governed reliability surface with explicit invariants.
- Changes to run-history behavior must keep mapped artifacts and suite contracts green.
