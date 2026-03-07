# Policy Contract

## Scope
Defines policy inputs, enforcement points, and decision visibility.

## Invariants
- Policy evaluation is deterministic for identical inputs.
- Deny decisions include a machine-readable reason.
- Debug mode may emit policy evaluation traces.

## Related tests
- `tests/e2e/policy/*`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Related schemas
- `configs/schema/dag.schema.json`

## Versioning and change policy
Policy input changes require schema and docs updates in the same change.
