# Documentation Governance and Truth Boundary

Status: accepted
Owner: platform documentation guild
Date: 2026-03-09

## Decision
Documentation is governed by strict source-of-truth boundaries. Normative contracts live in `docs/spec/`; explanatory and operational material must reference those contracts and must not duplicate them.

## Consequences
- Documentation drift checks are expected in control-plane governance.
- Root docs are entrypoints only.
- Contract text duplication is treated as governance debt.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-DOCUMENTATION-TRUTH-POLICY.md
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

### SOURCE: 20260309-DOCUMENTATION-GOVERNANCE-ALIGNMENT.md
# ADR renovation alignment

## Context

Repository cleanup introduced stronger boundaries for runtime scope, evidence governance, and test trust enforcement.

## Decision

- use a single docs/config governance policy to block inflation and unsupported claims
- require spec-to-code-and-test ownership mapping for normative contracts
- keep modeled/future behavior isolated from current implemented capability claims

## Consequences

- scorecard-style self-evaluation documents are removed from normative index surfaces
- docs-root and config inventory checks become release-gating governance controls
- future architectural changes must update governance mappings and reports in the same change set
