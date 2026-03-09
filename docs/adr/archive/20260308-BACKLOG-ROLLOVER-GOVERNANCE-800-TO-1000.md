# ADR: Backlog Rollover Governance from 800 to 1000

- Date: 2026-03-08
- Status: Accepted

## Context

The first 800 backlog items established governance foundations across fixtures, outputs, evidence, benchmarks, release gates, module hygiene, and documentation truth. The next backlog segment must preserve this governance shape and prioritize publish readiness.

## Decision

Rollover into the 1000-item plan follows these rules:

- preserve existing policy-first governance pattern
- require generated report + enforcing contract for each new governance domain
- prioritize execution order using high-impact and low-risk shortlists
- keep release-critical gates and docs-truth/module-hygiene gates green
- keep commit granularity small, logical, and reviewable

## Execution inputs

- `docs/reports/foundation/BACKLOG_HIGH_IMPACT_SHORTLIST_50_REPORT.md`
- `docs/reports/foundation/BACKLOG_LOW_RISK_HIGH_SIGNAL_SHORTLIST_50_REPORT.md`
- `docs/reports/foundation/BACKLOG_MAKE_TEST_PROMOTABLE_SHORTLIST_50_REPORT.md`
- `docs/reports/foundation/BACKLOG_V0_1_PUBLISH_READINESS_SHORTLIST_50_REPORT.md`
- `docs/reports/foundation/BACKLOG_DOCS_SITE_PUBLISH_READINESS_SHORTLIST_50_REPORT.md`
- `docs/reports/foundation/backlog_dependency_unlock_map_report.md`
- `docs/reports/foundation/DELIVERY_BOARD_1_800_REPORT.md`

## Consequences

- Backlog growth remains governed and auditable.
- Publish-readiness work remains tied to measurable reports and contracts.
- Rollover avoids ad-hoc expansion that bypasses established controls.
