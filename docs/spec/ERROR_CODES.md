# Error Codes

These codes are globally stable across bijux-dag versions.

## Errors
- `E1001` Duplicate node id
- `E1002` Dangling node reference
- `E1003` Dangling port reference
- `E1004` Cycle detected
- `E1005` JSON parse / unknown fields
- `E1006` Invalid spec version
- `E1007` Illegal node id characters
- `E1008` Output collision
- `E1009` Missing effects declaration
- `E1010` Env allowlist without env effect
- `E1011` Retry disallowed for nondeterministic effects
- `E1012` Invalid parameter reference
- `E1013` Effect denied by policy (network/env/clock)

## Warnings
- `W2001` Unreachable node
- `W2002` Orphan node
