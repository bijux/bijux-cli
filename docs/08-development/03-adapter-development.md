# Adapter Development

## Purpose
Define the adapter implementation contract and development workflow for backend integrations.

## Context
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

## Guarantees
- Adapter responsibilities and normalized output expectations are explicit.
- Capability gaps are documented rather than hidden.
- Adapter onboarding has a defined validation workflow.

## Limitations
- Backend platform constraints can limit feature parity.
- This document does not define all adapter-internal implementation details.
- Full portability still depends on shared capability surface between adapters.

## Related
- `docs/05-system-architecture/05-adapters.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/03-artifact-model.md`
