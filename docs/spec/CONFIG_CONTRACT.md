# Config Contract

## Scope
Defines precedence and behavior for CLI args, config files, environment, and defaults.

## Precedence
`CLI args > explicit config file > environment > defaults`.

Default baseline source: `configs/dev/default_runtime_config.json`.

## Invariants
- Unknown config fields are rejected unless explicitly marked for compatibility handling.
- Semantically equivalent config values normalize to equivalent internal config.

## Related tests
- `crates/bijux-dag-app/tests/cli_contract.rs`
- `tests/e2e/policy/*`

## Related schemas
- `configs/schema/runtime_config.schema.json`
- `configs/schema/policy_config.schema.json`

## Related docs
- `docs/spec/CONFIG_PRECEDENCE.md`
- `docs/spec/CONFIG_PRECEDENCE_CONTRACT.md`
- `docs/reference/CONFIG_INPUT_INVENTORY.md`
- `docs/spec/CONFIG_STATE_BOUNDARIES.md`
- `docs/spec/CONFIG_DEPRECATION.md`

## Versioning and change policy
Deprecated fields must include migration notes and validation behavior.
