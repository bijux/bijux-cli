# ADR: Internal Contract Discipline

## Status

Accepted

## Context

Internal contracts span multiple crates and boundaries. Missing ownership, fixture links, docs links, or suite mapping weakens reliability and maintainability.

## Decision

1. Require direct tests and ownership for internal contracts.
2. Require docs/spec linkage for stable internal contracts.
3. Maintain generated contract-to-fixture and contract-to-suite mapping outputs.
4. Enforce drift detection through dedicated governance suites.

## Consequences

- Internal contract quality becomes auditable and comparable over time.
- Boundary regressions are caught earlier through explicit governance signals.
- Maintainers get clear ownership and review expectations.

## Enforcement

- `configs/policy/internal_contract_governance.json`
- `configs/suites/internal_contract_verification.json`
- `crates/bijux-dev-dag/tests/internal_contract_governance_contracts.rs`
