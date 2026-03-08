# Internal Contract Discipline Policy

## Objective

Keep internal contracts explicit, directly tested, owned, and linked to fixtures/docs.

## Rules

1. Every internal contract must have a direct test and explicit owner.
2. Stable internal contracts must include docs/spec linkage.
3. Contract-to-fixture and contract-to-suite mappings must be generated and current.
4. Contract drift detection must run in governance suites.

## Enforcement

- `configs/policy/internal_contract_governance.json`
- `configs/suites/internal_contract_verification.json`
- `crates/bijux-dev-dag/tests/internal_contract_discipline_561_580_contracts.rs`
