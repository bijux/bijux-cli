# Backend Protocol Anomaly Report

## Scope

Protocol anomalies include negotiation mismatch, malformed payload handling, timeout propagation issues, ordering violations, and duplicate-ack inconsistencies.

## Detection anchors

- `crates/bijux-dev-dag/tests/remote_worker_protocol_contracts.rs`
- `crates/bijux-dev-dag/tests/remote_worker_protocol_release_contracts.rs`
- `crates/bijux-dag-runtime/tests/distributed_contracts.rs`

## Governance references

- `configs/suites/backend_protocol_verification.json`
- `evidence/cache/backend_protocol/regression_corpus.json`
