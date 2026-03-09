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
