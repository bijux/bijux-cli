# Release Gate and Readiness Policy

Status: accepted
Owner: release maintainers
Date: 2026-03-09

## Decision
Release gates are explicit, evidence-backed, and severity-aware; readiness dashboards are supporting references, not substitutes for contract checks.

## Consequences
- Release decisions remain reproducible and auditable.
- Advisory signals cannot override blocking contract failures.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-RELEASE-GATE-PHILOSOPHY.md
# ADR: Release Gate Philosophy

- Date: 2026-03-08
- Status: Accepted

## Context

Release gates existed across make targets and reports, but ownership, purpose, failure actions, and budget expectations were not governed from one policy surface.

## Decision

Adopt release gate governance policy at `configs/policy/release_gate_governance.json`.

Each governed gate must declare:

- owner
- purpose
- severity
- failure action
- docs page

Governance outputs include inventory, overlap/redundancy review, human and machine summaries, docs/workflow alignment checks, owner escalation map, runtime budget trend, and a compact release-review pack.

## Consequences

- Gate drift becomes contract-detectable.
- Contributors get a short quick-start for daily workflows.
- Maintainers get a deterministic triage flow and escalation map.
- Release review consumes a compact essential-output pack.

## Artifacts

- `configs/policy/release_gate_governance.json`
- `docs/reports/foundation/release_gate_inventory_report.md`
- `docs/reports/foundation/release_gate_human_summaries.md`
- `docs/reports/foundation/release_gate_machine_summaries.json`
- `docs/reports/foundation/release_review_pack.md`
- `docs/reference/RELEASE_GATE_CONTRIBUTOR_QUICKSTART.md`
- `docs/reference/RELEASE_GATE_MAINTAINER_TRIAGE_QUICKSTART.md`

### SOURCE: 20260308-DASHBOARDS-AND-READINESS-METRICS.md
# ADR: Dashboards and Readiness Metrics

## Status

Accepted

## Context

System quality signals exist across many reports, but operators and maintainers need a stable, consolidated readiness surface that is test-enforced.

## Decision

1. Maintain explicit dashboard pages for each major reliability and compatibility dimension.
2. Keep an overall readiness dashboard as a stable entrypoint.
3. Enforce dashboard presence and mapping through contract tests.
4. Require a verification suite that combines dashboard contracts with key subsystem contracts.

## Consequences

- Dashboard drift becomes visible and merge-blocking.
- Readiness review uses a single index instead of ad-hoc report discovery.
- Engineering ownership of readiness metrics stays explicit over time.

## Enforcement

- `crates/bijux-dev-dag/tests/system_readiness_dashboard_contracts.rs`
- `configs/suites/system_readiness_dashboards_verification.json`
