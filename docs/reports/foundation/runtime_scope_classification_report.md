# Runtime Scope Classification Report

Classification source: `configs/policy/runtime_module_lifecycle_status.json`.

| Lifecycle class | Intent |
| --- | --- |
| `core` | Required for local deterministic execution and canonical runtime semantics |
| `adapter` | Adapter integration surfaces required to execute supported node kinds |
| `operator-support` | Diagnostics/policy/support helpers that must not alter core execution identity |
| `experimental` | Non-default incubating surfaces under quarantine and expiration criteria |
| `speculative` | Modeled-only or broad surfaces that must remain quarantined |
