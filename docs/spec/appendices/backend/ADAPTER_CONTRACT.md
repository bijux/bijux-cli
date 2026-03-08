# Adapter contract

Adapter implementations must satisfy all requirements below.

## Scope
Defines adapter identity, capability metadata, execution behavior, and conformance requirements for built-in and external adapters.

## Identity and capabilities
- Stable adapter ID and version.
- Type-level origin classification: `BuiltIn` or `External`.
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

## Related tests
- `crates/bijux-dag-runtime/tests/adapter_conformance.rs`
- `crates/bijux-dag-runtime/tests/adapter_metadata_stability.rs`
- `tests/e2e/container/*`

## Versioning and change policy
Adapter contract changes must preserve existing descriptors or introduce explicit compatibility notes and conformance updates in the same change.
