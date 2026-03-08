# Backend Capability Query Reference

generated_from: `crates/bijux-dag-app/src/capability_matrix.rs`
format: `capabilities/v1`

## Query Surface

- `bijux dag capabilities --backend local --json`
- `bijux dag capabilities --backend kubernetes --json`
- `bijux dag capabilities --backend hpc --json`
- `bijux dag capabilities --backend remote --json`

## Response Stability Contract

- `backend` must remain one of: `local`, `kubernetes`, `hpc`, `remote`.
- `status` must remain `implemented` for `local` and `simulated` for `kubernetes`, `hpc`, and `remote`.
- unknown backend queries must return an unsupported-backend response surface.

## Evidence Links

- `crates/bijux-dag-app/src/capability_matrix.rs`
- `crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs`
- `crates/bijux-dev-dag/tests/k8s_adapter_release_contracts.rs`
- `crates/bijux-dev-dag/tests/hpc_adapter_release_contracts.rs`
- `crates/bijux-dev-dag/tests/remote_worker_protocol_release_contracts.rs`
