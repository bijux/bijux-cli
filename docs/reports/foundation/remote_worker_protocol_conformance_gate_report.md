# Remote Worker Protocol Conformance Gate Report

## Release gate

Remote backend claims are blocked unless all protocol conformance suites pass.

## Required fixtures

- `evidence/battle/fixtures/remote/simple_worker_pool.dag.json`
- `evidence/battle/fixtures/remote/fanout_many_small_nodes.dag.json`
- `evidence/battle/fixtures/remote/worker_protocol_failure_injection.json`
- `evidence/operator/fixtures/remote/worker_version_mismatch_explain.json`

## Required suites

- `crates/bijux-dag-runtime/tests/distributed_contracts.rs`
- `crates/bijux-dag-runtime/tests/remote_worker_protocol_conformance.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dev-dag/tests/remote_worker_protocol_contracts.rs`
- `crates/bijux-dev-dag/tests/remote_worker_protocol_release_contracts.rs`
