# Runtime semantics contract

## Scope

This contract defines core runtime semantics for scheduling, retries, failures, cache behavior, replay, and audit traces.

## Deterministic scheduling

- Ready nodes must be ordered deterministically.
- Tie-break order must be stable for identical priority.
- Fairness must prevent indefinite starvation.

## Node execution semantics

- Retry eligibility is bounded by explicit max attempts.
- Timeout is terminal for the current attempt.
- Cancellation is terminal once node terminal state is reached.
- Dependency resolution requires all required upstream nodes.

## Artifact and cache semantics

- Artifact commit requires complete write and manifest synchronization.
- Cache reuse requires fingerprint, schema, and proof metadata.
- Cache invalidation is required on policy, adapter version, or output schema change.

## Replay and manifest semantics

- Replay equivalence is validated by semantic fingerprint parity.
- Run manifest validity requires run header, trace index, outputs index, and consistent totals.

## Recovery, lineage, and failure semantics

- Recovery is required when checkpoints exist without terminal completion.
- Artifact lineage must exist for all referenced outputs.
- Failure classification must be explicit and machine-readable.

## Runtime audit and trace semantics

- Runtime emits append-only audit events.
- Trace event categories are countable by stable category key.
