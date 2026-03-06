# Work-stealing scheduler boundaries

## Current boundaries

- Planning boundary: `ExecutionPlan` is produced before execution.
- Scheduling boundary: `Scheduler` selects ready nodes based on policy and runtime state.
- Executor boundary: `LocalExecutor` handles bounded submission and in-flight tracking.

## Future work-stealing design

- Introduce per-worker ready deques with global fallback queue.
- Preserve deterministic mode as the correctness baseline.
- Add a policy switch between deterministic and work-stealing runtime modes.
- Maintain stable event semantics independent of scheduler implementation.

## Contract requirements before implementation

- `SchedulerEventHook` coverage for eligible, blocked, and scheduled transitions.
- Queue isolation policy surface remains unchanged (`SingleQueue`, `GroupIsolated`).
- Execution checkpoints keep the same schema across scheduler implementations.
