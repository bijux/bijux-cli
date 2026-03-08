# ADR: Documentation Truth Policy

- Date: 2026-03-08
- Status: Accepted

## Context

Documentation drift risk increased as governance, evidence, benchmark, and support surfaces expanded. We need a single policy that links docs pages to owners, code paths, tests, and canonical generated evidence sources.

## Decision

Adopt docs truth governance policy at `configs/policy/docs_truth_governance.json`.

Required rules:
- docs for speculative/modelled surfaces must state that boundary explicitly
- docs for stable/shipped surfaces must link concrete evidence
- docs truth drift is enforced by `make docs-truth-drift`

Required checks:
- mission and positioning wording drift
- support matrix drift versus generated backend capability data
- evidence docs drift versus evidence governance inventory
- benchmark docs drift versus benchmark governance registry

## Consequences

- documentation drift becomes release-gate detectable
- docs ownership and code/test linkage are explicit
- stale and oversell docs become trackable reports rather than ad-hoc reviews
