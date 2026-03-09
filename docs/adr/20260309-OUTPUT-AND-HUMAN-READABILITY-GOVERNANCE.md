# Output and Human Readability Governance

Status: accepted
Owner: operator UX maintainers
Date: 2026-03-09

## Decision
Human-facing outputs are concise, stable, and aligned with machine-readable contracts.

## Consequences
- Human wording remains deliberate and deterministic.
- Operator surfaces avoid unstable or speculative messaging.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-HUMAN-OUTPUT-PHILOSOPHY-AND-STABILITY.md
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

- Keep `cargo run -p bijux-dev-dag --bin generate_human_output_governance_reports` in governance workflows.
- Treat non-zero human-output snapshot gaps as release blockers.

### SOURCE: 20260308-STABLE-OPERATOR-SURFACE.md
# ADR: Stable Operator Surface

## Status

Accepted

## Context

Operator command surfaces accumulated overlapping outputs and multiple ways to answer similar operational questions. This increased cognitive load and made stable usage harder to explain.

## Decision

1. Maintain a compact canonical operator command set for default workflows.
2. Keep detailed output opt-in while preserving concise defaults.
3. Keep JSON output available for core automation-facing commands.
4. Treat experimental or modeled terms as non-default and clearly marked.
5. Enforce operator surface stability through dedicated verification suite and contracts.

## Consequences

- Operator workflows become easier to teach and automate.
- Redundant command stories are reduced without removing needed diagnostic depth.
- Surface drift is detected via contract and snapshot tests.

## Enforcement

- `configs/suites/operator_surface_verification.json`
- `crates/bijux-dev-dag/tests/operator_surface_guarantees_contracts.rs`
