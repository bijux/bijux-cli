# Replay guarantees

## Replay contract

- Replay uses the embedded graph snapshot from the source run.
- Replay output path is isolated from source unless explicitly shared.
- Equivalent graph fingerprint and canonical graph representation are required for equivalence checks.

## Behavioral guarantees

- Replayed runs preserve node ordering and outputs for deterministic graphs.
- Node status transitions remain stable under the same policy, selector set, and timeout/cost constraints.
- Replay failures are captured in run traces and can be compared using existing diff contracts.
