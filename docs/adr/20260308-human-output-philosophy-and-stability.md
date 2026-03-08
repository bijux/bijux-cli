# ADR: Human Output Philosophy and Stability

- Date: 2026-03-08
- Status: Accepted

## Context

Operator-facing text output is a product contract. It must be understandable, stable for tooling/documentation, and explicit about supported behavior without overclaiming modeled or speculative surfaces.

## Decision

Adopt governed human-output policy at `configs/policy/human_output_governance.json` with mandatory expectations:

- Snapshot protection for every stable command family.
- Concise and detailed examples for every governed family.
- Generated inventories and gap reports for snapshot coverage.
- Wording drift report for equivalent command surfaces.
- Freshness gate requiring zero missing snapshot surfaces.
- Default human output must not use speculative/modeled terminology.

## Consequences

Positive:

- Human-output drift becomes measurable and release-gated.
- Operator UX references are generated from current snapshots/examples.
- Documentation and CLI output stay synchronized.

Tradeoff:

- Snapshot/example refresh is required whenever wording intentionally changes.

## Follow-up

- Keep `tools/generate_human_output_governance_reports.sh` in governance workflows.
- Treat non-zero human-output snapshot gaps as release blockers.
