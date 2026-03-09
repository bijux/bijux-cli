# Exit Codes Used in Tests

## Source of truth
- `tests/unit/core/test_exit_policy.py`
- `tests/unit/core/test_bootstrap_flow.py`
- command/unit/regression assertions across `tests/`

## Contracted exit codes asserted
- `0` success
- `1` internal/general failure
- `2` usage or user-input failure
- `3` ASCII/encoding or serialization failure
- `130` user abort/signal interruption

## Additional observed test-only values
- `42` appears in targeted unit testing for explicit non-contract passthrough behavior (`tests/unit/cli/commands/memory/test_memory.py`).
