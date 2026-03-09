# Backend Protocol Coverage Report

## Coverage matrix

| Coverage class | Anchor |
| --- | --- |
| handshake and version negotiation | `docs/spec/WORKER_PROTOCOL_CONTRACT.md` |
| compatibility and error propagation | `crates/bijux-dev-dag/tests/remote_worker_protocol_contracts.rs` |
| timeout and retry logic | `docs/spec/WORKER_PROTOCOL_CONTRACT.md`, `crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs` |
| message ordering and replay safety | `crates/bijux-dag-runtime/tests/distributed_contracts.rs`, `docs/spec/REPLAY_CONTRACT.md` |
| serialization/corruption detection | `evidence/cache/distributed_execution/regression_corpus.json` |
| stress, benchmarks, telemetry, fuzzing | `configs/suites/distributed_execution_stress.json`, `docs/reports/foundation/DISTRIBUTED_EXECUTION_BENCHMARKS.md`, `docs/spec/RUNTIME_TELEMETRY_SCHEMA.md`, `evidence/cache/determinism/regression_corpus.json` |

## Completion signals

- contract: `docs/spec/BACKEND_PROTOCOL_STABILITY_CONTRACT.md`
- suite: `configs/suites/backend_protocol_verification.json`
- corpus: `evidence/cache/backend_protocol/regression_corpus.json`
