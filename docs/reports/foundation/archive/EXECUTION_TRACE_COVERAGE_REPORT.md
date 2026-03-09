# Execution Trace Coverage Report

## Coverage map

| Coverage class | Anchor |
| --- | --- |
| node start/completion | `crates/bijux-dag-app/src/lib.rs` |
| scheduler decisions | `docs/spec/SCHEDULER_CONTRACT.md` |
| artifact reads/writes | `docs/spec/STORAGE_CONTRACT.md` |
| replay and cache decisions | `crates/bijux-dag-app/src/replay/service.rs` |
| backend dispatch and worker communication | `docs/spec/DISTRIBUTED_EXECUTION_ARCHITECTURE_CONTRACT.md` |
| serialization and schema | `docs/spec/NODE_TRACE_SCHEMA_V0.1.md` |
| corruption and restart persistence | `docs/spec/RUN_DIR_CONTRACT.md` |

## Completion signals

- contract: `docs/spec/EXECUTION_TRACE_RECORDS_CONTRACT.md`
- suite: `configs/suites/execution_trace_regression.json`
- corpus: `evidence/cache/execution_trace/regression_corpus.json`
