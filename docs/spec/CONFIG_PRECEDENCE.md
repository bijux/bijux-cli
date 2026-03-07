# Configuration Precedence

## Scope
Defines the single precedence table for effective configuration resolution.

## Precedence
`CLI > explicit config file > environment > defaults`

## Notes
- CLI values override all lower layers when provided.
- Explicit config file values override environment/default values.
- Environment values are only used for fields that are contractually env-addressable.
- Defaults are applied only when no higher layer provides a value.

## Related tests
- `crates/bijux-dag-app/tests/config_precedence_contract.rs`
- `crates/bijux-dag-app/tests/config_validation_contract.rs`

## Versioning and change policy
Any precedence change is breaking and requires docs + tests + drift-check updates in one change.
