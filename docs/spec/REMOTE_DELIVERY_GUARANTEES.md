# Remote Delivery Guarantees

## Scope

Defines delivery guarantees and reliability boundaries for remote/distributed worker protocol execution.

## Delivery guarantees

- Dispatch and status delivery are **at-least-once**.
- Exactly-once execution is not guaranteed by transport alone.
- Duplicate dispatch and duplicate acknowledgements must be handled by protocol dedup keys.

## Hard guarantees

- Lease expiration and recovery windows are explicit and machine-checkable.
- Heartbeat classification (`healthy`, `delayed`, `lost`) is deterministic.
- Status event ordering is normalized by `sequence` before run-record projection.
- Artifact commit visibility requires successful upload+commit binding.

## Best-effort guarantees

- Status reporting latency under network partition.
- Cancellation delivery latency in congested control links.
- Worker reconnect timing after crash or process restart.

## Operator implications

- Operators should treat delayed heartbeat as a degraded state, not failure completion.
- Replay and diff remain authoritative for final semantic outcomes.
