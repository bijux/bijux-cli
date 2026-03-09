# Run History Integrity Guarantees

Status: accepted
Owner: runtime maintainers
Date: 2026-03-09

## Decision
Run history is treated as authoritative operational evidence with strict integrity and corruption-recovery behavior.

## Consequences
- Run identity and ancestry remain stable surfaces.
- Corruption handling must not panic and must stay diagnosable.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-RUN-HISTORY-GUARANTEES.md
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
  - `docs/reports/foundation/RUN_HISTORY_221_240_STATUS_REPORT.md`
- Size and resilience reports:
  - `docs/reports/foundation/RUN_HISTORY_SIZE_GROWTH_REPORT.md`
  - `docs/reports/foundation/RUN_HISTORY_CORRUPTION_RESILIENCE_REPORT.md`
- Diagnostics and consistency:
  - `docs/reports/foundation/RUN_HISTORY_DIAGNOSTICS_REPORT.md`
  - `docs/reports/foundation/RUN_HISTORY_CONSISTENCY_DASHBOARD.md`
- Verification suite:
  - `configs/suites/run_history_invariants.json`

## Consequences

- Run history becomes a governed reliability surface with explicit invariants.
- Changes to run-history behavior must keep mapped artifacts and suite contracts green.
