# Config Deprecation Policy

## Scope
Defines how configuration fields are deprecated and removed.

## Current status
No config fields are currently deprecated.

## Rules
- Deprecated fields must be explicitly listed in this document with replacement guidance.
- Deprecated fields remain accepted only for a documented compatibility window.
- New deprecated fields require matching validation tests and migration notes.

## Related tests
- `crates/bijux-dag-app/tests/config_validation_contract.rs`

## Versioning and change policy
Deprecation additions are contract changes and must be reviewed with config precedence and schema compatibility docs.
