# Config Key/Value Parity Coverage

This document records completion for stable compatibility coverage.

## Implemented behavior

- Key normalization and validation are enforced in `crates/bijux-cli/src/config/validation.rs`.
- Value validation is enforced in the same module and consumed by config command execution.
- Error normalization maps key/value usage failures to usage exit code semantics.

## Test coverage mapping

### Keys (61-71)

- Empty key: covered in unit + integration tests.
- Whitespace-only key: covered in unit + integration tests.
- Lowercase keys: covered.
- Mixed-case keys: covered.
- Underscore-only keys: covered.
- Alphanumeric keys: covered.
- Invalid punctuation keys: covered.
- Dots in keys: covered.
- Dashes in keys: covered.
- Non-ASCII keys: covered.

### Values (72-80)

- Plain ASCII values: covered.
- Quoted values: covered.
- Escaped quoted values: covered.
- Empty values: covered.
- Values with spaces: covered.
- Newline rejection: covered.
- Tab rejection: covered.
- Control-character rejection: covered.

## Sources

- `crates/bijux-cli/src/config/validation.rs`
- `crates/bijux-cli/tests/integration/cli/config/config_key_value_parity.rs`
- `crates/bijux-cli/tests/integration/cli/config/config_parity.rs`
