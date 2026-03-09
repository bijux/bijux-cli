# Adapter Development

Define the adapter implementation contract and development workflow for backend integrations.

Adapters bridge runtime intent to backend-specific execution systems and directly affect portability guarantees.

## Explanation
Adapter interface responsibilities:
- receive normalized runtime work units.
- execute work in backend-specific environment.
- return normalized result envelope (status, output references, diagnostics).
- preserve cancellation/timeout semantics where capability exists.

Adapter development rules:
- map backend-native states to canonical run/node states.
- surface unsupported features explicitly; do not emulate silently unless documented.
- preserve evidence completeness needed for inspect/replay/diff.

Contract checklist for new adapters:
- run lifecycle support validated.
- timeout and cancellation behavior documented.
- artifact lineage attribution preserved.
- replay/diff compatibility limitations documented.
- support tier declared (`stable`, `provisional`, `experimental`).

Validation workflow:
1. implement adapter against runtime boundary interface.
2. run adapter-focused integration tests and fixture scenarios.
3. verify replay/diff behavior against baseline backend.
4. publish support tier and capability notes.

## Examples
```text
Adapter normalization example:
backend exit: code 137
canonical node status: failed_timeout_or_kill (classified)
diagnostic fields: backend_reason, stderr_excerpt, duration_ms
```

```text
Capability declaration sample:
supports_timeout: true
supports_cancel: true
artifact_streaming: false
tier: provisional
```

```rust
// Conceptual adapter result mapping example
fn map_backend_exit(exit_code: i32) -> NodeOutcome {
    match exit_code {
        0 => NodeOutcome::Succeeded,
        124 => NodeOutcome::FailedTimeout,
        _ => NodeOutcome::FailedNonZero(exit_code),
    }
}
```

## Guarantees
- Adapter responsibilities and normalized output expectations are explicit.
- Capability gaps are documented rather than hidden.
- Adapter onboarding has a defined validation workflow.
- Includes implementation-oriented mapping example for adapter authors.

## Limitations
- Backend platform constraints can limit feature parity.
- This document does not define all adapter-internal implementation details.
- Full portability still depends on shared capability surface between adapters.

## Related
- `docs/05-system-architecture/05-adapters.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/03-artifact-model.md`

## Adapter lifecycle and failure-case handling

Adapter lifecycle:

1. declare capability model and support tier,
2. implement runtime interface mapping,
3. validate normalization against fixture-backed scenarios,
4. verify replay/diff behavior against baseline adapter,
5. publish documented limitations and maintenance owner.

Failure cases adapter authors must handle explicitly:

- backend timeout/cancel mismatch,
- missing artifact write acknowledgment,
- partial execution completion with ambiguous exit reason,
- backend error payloads that do not map directly to canonical outcomes.

## Worked end-to-end adapter example

Example flow for a new `container-x` adapter:

1. implement work-unit execution wrapper,
2. map container exit + diagnostics to canonical node outcomes,
3. emit artifact lineage references into run evidence,
4. run integration fixtures comparing `container-x` to stable `local-shell`,
5. classify capability gaps and mark tier `provisional` until parity checks pass.

## What adapters may decide versus must not redefine

Adapters may decide:

- backend invocation mechanics,
- backend-specific diagnostics collection,
- capability declaration values.

Adapters must not redefine:

- DAG semantics,
- run/artifact identity contracts,
- diff/replay classification vocabulary.
