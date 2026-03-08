# ADR: Output and Schema Governance End-State

- Date: 2026-03-08
- Status: Accepted

## Context

Stable JSON command outputs must remain contract-safe for operators and automation. Existing schema references, command mappings, and lockstep tests were distributed and hard to audit for completeness.

## Decision

Adopt one governed source for stable JSON output coverage:

- Policy: `configs/policy/json_output_governance.json`
- Generated evidence:
  - command-to-schema inventory
  - schema-to-command-and-lockstep inventory
  - missing example report
  - missing lockstep report
  - schema registry page
  - stable JSON command registry page
- Required artifacts per schema:
  - minimal example output
  - maximal example output
  - lockstep test marker

## Consequences

Positive:

- Missing JSON contract artifacts are visible and release-gated.
- Schema and output ownership remain explicit and audit-friendly.
- Freshness checks are deterministic and easy to regenerate.

Tradeoff:

- Governance policy and generated docs must be refreshed when stable JSON surfaces change.

## Follow-up

- Keep `tools/generate_json_output_governance_reports.sh` in release workflows.
- Treat non-zero gap report counts as blocking until resolved.
