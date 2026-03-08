# Runtime Fault Tolerance Coverage Report

## Coverage matrix

| Coverage class | Anchor |
| --- | --- |
| crash recovery and restart continuation | `crates/bijux-dag-runtime/tests/runtime_recovery_contracts.rs` |
| scheduler restart and restart determinism | `crates/bijux-dag-runtime/tests/runtime_scheduler_state_machine_invariants_contracts.rs` |
| worker reconnect and event-log recovery | `docs/spec/WORKER_PROTOCOL_CONTRACT.md`, `docs/spec/CONCURRENCY_MODEL.md` |
| replay/cancellation/partial-run recovery | `crates/bijux-dag-app/tests/fault_resilience_integration.rs` |
| failure detection and injection | `docs/spec/FAILURE_TAXONOMY_CONTRACT.md`, `configs/suites/failure_recovery_injection_stress.json` |
| resilience benchmarks and telemetry | `docs/reports/foundation/failure_handling_benchmarks.md`, `docs/spec/RUNTIME_TELEMETRY_SCHEMA.md` |

## Completion signals

- contract: `docs/spec/RUNTIME_FAULT_TOLERANCE_CONTRACT.md`
- suite: `configs/suites/runtime_fault_tolerance_verification.json`
- corpus: `evidence/cache/runtime_fault_tolerance/regression_corpus.json`
