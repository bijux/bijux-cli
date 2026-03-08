# Runtime State-Machine Cancellation Trace Fixtures

Generated: 2026-03-08

## Fixture Corpus

- `cancel_before_dispatch`: cancellation requested before scheduler admission.
- `cancel_during_running`: cancellation requested with active running nodes.
- `cancel_after_partial_completion`: cancellation after a subset reaches terminal states.
- `cancel_with_retry_queue`: cancellation while retry queue is non-empty.

## Required Trace Fields

- `event`
- `ts`
- `node_id` (when node-scoped)
- `run_id`
- `cause`

## Contract Invariants

- Timestamps are monotonic for each run trace.
- No terminal node transitions back to non-terminal states.
- Run terminal state is reachable from `running` through legal transitions only.
- Cancellation does not bypass trace emission for terminal outcomes.
