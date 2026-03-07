# Error Contract

## Scope
Defines error classes, stable machine-readable fields, and exit-code policy for user-facing failures.

## Error classes
- parse
- schema
- validation
- config
- policy
- execution
- io
- replay
- cache
- compatibility
- internal

## Invariants
- JSON output includes stable code and class fields.
- Human-readable output prioritizes exact cause and action guidance.

## Related tests
- `crates/bijux-dag-app/tests/output_contract.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Versioning and change policy
Public error code additions require docs plus test coverage in the same change.
