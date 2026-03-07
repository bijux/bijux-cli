# Test debt ledger

## Intentional gaps

- Full top-level `tests/e2e` matrix runner is not yet complete.
- Fault-class simulation for disk-full uses limited deterministic simulation.
- Performance and resource trend gating is warning-level, not release-blocking.

## Why gaps remain

- control-plane and testkit extraction landed first
- benchmark/resource governance is being introduced incrementally

## Exit criteria

- each listed gap must map to an issue and target release window
- removal from this ledger requires linked test path and control-plane gate
