# Scheduler state-space contract

State-space constraints:
- Node transitions must follow formal node state machine legality.
- Run transitions must follow formal run state machine legality.
- Illegal transitions are contract-test failures.

Determinism constraints:
- Dispatch outcome for deterministic workloads is independent of thread count.
- Failure propagation and retry sequencing are replay-explainable.

Policy constraints:
- `clean_env` and `deny_env` interactions are deterministic.
- `deny_network` behavior must be enforced consistently across shell and container adapters.
