# Canonical Fixture Strategy Policy

## Objective

Reduce fixture sprawl by preferring canonical reusable fixtures and explicit governance tags.

## Rules

1. Fixture families must be governed by owner, suite, and purpose.
2. Fixtures must use one of: `canonical`, `stress`, `corrupt`, `smoke`, `legacy`.
3. Smoke defaults may only use `canonical` or `smoke` tags.
4. Orphan and duplicate fixtures are cleanup candidates and must be tracked.

## Enforcement

- `configs/policy/fixture_family_governance.json`
- `configs/suites/fixture_contraction_verification.json`
- `crates/bijux-dev-dag/tests/fixture_contraction_521_540_contracts.rs`
