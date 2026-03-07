# Backend contract

## Scope

This contract is the normative source for execution backend lifecycle, capability binding, and conformance requirements.

## Lifecycle

Each backend implementation must implement this lifecycle in order:

1. `prepare`
2. `launch`
3. `observe`
4. `finalize`
5. `cleanup`

`cleanup` must execute after both successful and failed lifecycle paths.

## Backend API boundary

- `ExecutionBackend` is the only runtime backend interface.
- Runtime orchestration must call lifecycle hooks through this interface only.
- Backend binding must fail before execution when required capabilities are incompatible.

## Backend classes

- `FakeBackend`: deterministic backend for engine tests.
- `ProcessLikeBackend`: local subprocess-like backend model.
- Container and remote models are separate contracts and are not mixed into local process lifecycle.

## Capability descriptor

Every backend reports stable capability fields:

- `backend_name`
- `kind`
- `supports_env_shaping`
- `supports_timeout`
- `supports_stream_capture`

## Conformance requirements

Required conformance coverage includes:

- fake/process parity behavior
- prepare failure classification
- launch failure classification
- observe timeout classification
- cleanup on success
- cleanup on failure
- explicit environment shaping behavior
- undeclared output rejection
- registry report coverage

## Governance

- `bijux-dev-dag repo --domain governance --suite backend-contract` is required for backend changes.
- New backend implementations are blocked until backend contract conformance remains explicit and passing.
