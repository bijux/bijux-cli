# Remote Worker Proof Example

## Scenario

- run_id: `run-remote-proof-001`
- backend_origin: `remote-run`
- graph_id: stable against canonical graph snapshot

## Evidence links

- worker protocol conformance: `crates/bijux-dag-runtime/tests/remote_worker_protocol_conformance.rs`
- import/export replay contract: `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- capability surface: `bijux dag capabilities --backend remote --json`

## Proof interpretation

The run is reproducible at protocol-contract level when:

- lease/heartbeat semantics pass conformance suite
- event-ordering and duplicate-ack handling pass
- replay from imported bundle preserves semantic output equivalence

## Trust boundary

This proof is unsigned and contract-level; it demonstrates deterministic behavior in simulated remote mode only.
