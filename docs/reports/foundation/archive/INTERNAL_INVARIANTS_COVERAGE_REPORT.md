# Internal Invariants Coverage Report

## Coverage matrix

| Coverage class | Anchor |
| --- | --- |
| graph/planner/runtime/scheduler invariants | `docs/spec/FORMAL_INVARIANTS.md`, `runtime_core/governance/invariants.rs` |
| artifact store and run history invariants | `docs/spec/STORAGE_CONTRACT.md`, `docs/spec/RUN_HISTORY_CONTRACT.md` |
| invariant violation detection and logging | `runtime_core/execution/engine.rs` |
| telemetry and anomaly detection | `docs/spec/RUNTIME_TELEMETRY_SCHEMA.md` |
| stress and fuzz checks | `configs/suites/runtime_engine_scheduler_fast.json`, `validation_contracts_21_40.rs` |
| debugging and trace capture | `runtime_core/execution/scheduler.rs`, `docs/spec/SCHEDULER_CONTRACT.md` |

## Completion signals

- contract: `docs/spec/INTERNAL_INVARIANTS_CONSISTENCY_CONTRACT.md`
- suite: `configs/suites/internal_invariants_verification.json`
- corpus: `evidence/cache/invariants/regression_corpus.json`
