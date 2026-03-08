# ADR: Evidence Minimalism

## Status

Accepted

## Context

Evidence surfaces expanded in breadth and overlap, reducing decision clarity and increasing maintenance burden.

## Decision

1. Each evidence family must declare severity, audience, source-of-truth, and release-review relevance.
2. Duplicate or low-value evidence outputs should be merged into canonical decision surfaces.
3. Release-critical and advisory evidence paths must remain isolated in governance behavior.
4. Compact evidence index and claim mapping are required for operator and maintainer clarity.

## Consequences

- Evidence decision value is easier to evaluate.
- Governance can block low-signal evidence growth.
- Release review consumes a smaller and clearer evidence set.

## Enforcement

- `configs/policy/evidence_family_governance.json`
- `configs/suites/evidence_signal_sharpening_verification.json`
- `crates/bijux-dev-dag/tests/evidence_signal_quality_contracts.rs`
