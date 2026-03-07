# Spec to code and test ownership

## Scope

This mapping binds normative specs to owning code paths and owning test suites.

| spec | owning code path | owning test suite |
| --- | --- | --- |
| `docs/spec/EXECUTION_SEMANTICS_CONTRACT.md` | `crates/bijux-dag-runtime/src/runtime_core` | `crates/bijux-dag-runtime/tests/runtime_semantics_contracts.rs` |
| `docs/spec/SCHEDULER_CONTRACT.md` | `crates/bijux-dag-runtime/src/runtime_core/scheduler.rs` | `crates/bijux-dag-runtime/tests/runtime_scheduler_contracts.rs` |
| `docs/spec/STATE_MACHINE_CONTRACT.md` | `crates/bijux-dag-runtime/src/state_machine` | `crates/bijux-dag-runtime/tests/state_machine_transitions.rs` |
| `docs/spec/CACHE_CONTRACT.md` | `crates/bijux-dag-runtime/src/cache` | `crates/bijux-dag-runtime/tests/runtime_cache_contracts.rs` |
| `docs/spec/REPLAY_CONTRACT.md` | `crates/bijux-dag-runtime/src/replay` | `crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs` |
| `docs/spec/IMPORT_EXPORT_CONTRACT.md` | `crates/bijux-dag-app/src/import_export` | `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs` |
| `docs/spec/OPERATOR_UX_CONTRACT.md` | `crates/bijux-dag-app/src/inspect` | `crates/bijux-dag-app/tests/operator_ux_contract.rs` |
| `docs/spec/CRATE_BOUNDARY_CONTRACT.md` | `crates/bijux-dev-dag/src/commands` | `crates/bijux-dev-dag/tests/crate_taxonomy_guardrails.rs` |
| `docs/spec/EVIDENCE_MODEL.md` | `crates/bijux-dev-dag/src/commands` | `crates/bijux-dev-dag/tests/evidence_governance_contract.rs` |
| `docs/spec/TEST_TRUST_LEDGER.md` | `configs/policy/test_trust_ledger.json` | `crates/bijux-dev-dag/tests/test_trust_cleanup_contracts.rs` |

## Rule

Every normative spec in `docs/spec/` must remain mapped to one code path and one test suite.
