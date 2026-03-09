# Dual Install Behavior

When `pip` and `cargo` installations coexist, runtime selection is deterministic and contract-driven.

## Resolution Rule

1. If `BIJUX_BIN` is set, that absolute path is the active runtime.
2. Otherwise, the first `bijux` binary found in `PATH` order is active.

## Compatibility Rule

- Dual installs are supported only when they resolve to the same major-version command contract.
- Mixed-major binaries in `PATH` are treated as ambiguous and should be fixed before automation use.

## Operational Guidance

- Run `bijux cli paths` to see the active binary and all discovered `bijux` binaries in `PATH` order.
- Run `bijux cli doctor` to detect path shadowing, duplicate ecosystem installs, stale wrappers, and wheel/runtime version mismatches.

