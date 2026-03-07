# Adapter contract

Adapter implementations must satisfy all requirements below.

## Identity and capabilities
- Stable adapter ID and version.
- Declared supported node kinds.
- Declared required effects.
- Declared output schema version.

## Execution contract
- Inputs are materialized only from declared upstream dependencies.
- Outputs must be declared and indexed; undeclared output writes are failures.
- Failures must be classified with stable machine codes.
- stdout/stderr capture must be persisted deterministically.
- Timeout and cancellation behavior must map to explicit runtime status.

## Environment contract
- Environment exposure is deny-by-default.
- Allowed environment variables must be explicit.
- Hermetic mode forbids undeclared environment and network access.

## Conformance
Every adapter must pass the runtime adapter conformance suite and metadata reproducibility checks across run and replay.
