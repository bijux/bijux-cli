# Runtime Scope Governance Policy

## Lifecycle classes

Every runtime module must declare one lifecycle class:

- `core`
- `adapter`
- `operator-support`
- `experimental`
- `speculative`

## Required controls

1. New runtime modules require explicit lifecycle declaration before merge.
2. `experimental` and `speculative` runtime modules require explicit expiration criteria.
3. `experimental` and `speculative` runtime modules must remain quarantined from default operator surfaces.
4. Quarantined modules must not be presented as stable capability guarantees.

## Enforcement

- `configs/policy/runtime_module_lifecycle_status.json`
- `crates/bijux-dev-dag/tests/runtime_scope_contraction_401_420_contracts.rs`
- `configs/suites/runtime_scope_contraction_verification.json`
