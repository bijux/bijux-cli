# Config Contract

## Scope
Defines precedence and behavior for CLI args, config files, environment, and defaults.

## Precedence
`CLI args > explicit config file > environment > defaults`.

## Invariants
- Unknown config fields are rejected unless explicitly marked for compatibility handling.
- Semantically equivalent config values normalize to equivalent internal config.

## Related tests
- `crates/bijux-dag-app/tests/cli_contract.rs`
- `tests/e2e/policy/*`

## Related schemas
- `configs/schema/dag.schema.json`
- `configs/schema/dag.schema.json`

## Versioning and change policy
Deprecated fields must include migration notes and validation behavior.
