# Evidence claim links

## Scope

Links high-level claims to concrete test and contract surfaces.

| claim | contract | evidence |
| --- | --- | --- |
| deterministic scheduling is enforced | `docs/spec/SCHEDULER_CONTRACT.md` | `crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs` |
| replay detects semantic drift | `docs/spec/REPLAY_CONTRACT.md` | `crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs` |
| artifact integrity is guarded | `docs/spec/ARTIFACT_INTEGRITY_SUITE.md` | `crates/bijux-dag-runtime/tests/runtime_artifact_contracts.rs` |
| runtime state transitions are legal-only | `docs/spec/STATE_MACHINE_CONTRACT.md` | `crates/bijux-dag-runtime/tests/state_machine_transitions.rs` |
| import/export preserves provenance | `docs/spec/IMPORT_EXPORT_CONTRACT.md` | `crates/bijux-dag-runtime/tests/runtime_import_export_contracts.rs` |
| operator inspection surfaces remain stable | `docs/spec/OPERATOR_UX_CONTRACT.md` | `crates/bijux-dag-app/tests/operator_ux_contract.rs` |
