# Runtime Adapter Registry Coverage Dashboard

Generated: 2026-03-08

Scoped files:
- `crates/bijux-dag-runtime/src/adapters/adapter.rs`
- `crates/bijux-dag-runtime/src/adapters/registry.rs`
- `crates/bijux-dag-runtime/src/adapters/runtime_registry.rs`
- `crates/bijux-dag-runtime/src/backend/capability.rs`
- `crates/bijux-dag-runtime/src/backend/contract.rs`

Direct coverage anchors:
- `crates/bijux-dag-runtime/tests/adapter_registry_capability_contracts.rs`
- `crates/bijux-dag-runtime/tests/backend_capability_boundary_contracts.rs`
- `crates/bijux-dag-runtime/tests/backend_contract.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`

Key verified behaviors:
- Duplicate adapter identity and duplicate kind registration rejection.
- Deterministic registry listing and deterministic adapter selection tie-break.
- Capability query stability for shipped backend surfaces.
- Unknown backend queries reject unsupported names.
- Backend contract mismatch failures stay explicit and typed.
