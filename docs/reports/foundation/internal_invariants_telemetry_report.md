# Internal Invariants Telemetry Report

## Telemetry scope

- invariant violation counters by invariant id
- invariant failure logging signal counters
- anomaly counters tied to invariant failures
- scheduler and runtime invariant signal coverage

## Source anchors

- `docs/spec/RUNTIME_TELEMETRY_SCHEMA.md`
- `crates/bijux-dag-runtime/src/diagnostics/runtime/operations_governance.rs`
- `evidence/cache/invariants/regression_corpus.json`
