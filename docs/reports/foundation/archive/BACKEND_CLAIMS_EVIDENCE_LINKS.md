# Backend Claims with Evidence Links

generated_from: `docs/reports/foundation/adapter_conformance_coverage_matrix.json`

## Claims

- kubernetes capability contracts are simulated and evidence-backed:
  - `crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs`
  - `crates/bijux-dev-dag/tests/k8s_adapter_release_contracts.rs`
- hpc capability contracts are simulated and evidence-backed:
  - `crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs`
  - `crates/bijux-dev-dag/tests/hpc_adapter_release_contracts.rs`
- remote capability contracts are simulated and evidence-backed:
  - `crates/bijux-dag-runtime/tests/remote_worker_protocol_conformance.rs`
  - `crates/bijux-dev-dag/tests/remote_worker_protocol_release_contracts.rs`

## Release Gate

- backend capability query docs are generated artifacts and must include `generated_from`.
