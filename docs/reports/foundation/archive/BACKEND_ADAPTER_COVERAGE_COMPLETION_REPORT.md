# Backend and Adapter Coverage Completion Report (321-340)

This report maps TODO 321-340 to direct runtime tests, generated evidence artifacts, and release governance gates.

## 321-325 direct module coverage

- `runtime/src/adapters/adapter.rs`
- `runtime/src/adapters/registry.rs`
- `runtime/src/adapters/runtime_registry.rs`
- `runtime/src/backend/capability.rs`
- `runtime/src/backend/contract.rs`

Coverage anchors:
- `crates/bijux-dag-runtime/tests/adapter_registry_capability_contracts.rs`
- `crates/bijux-dag-runtime/tests/backend_capability_boundary_contracts.rs`
- `crates/bijux-dag-runtime/tests/backend_contract.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`

## 326-335 behavior contracts

- duplicate adapter rejection
- incomplete capability declaration rejection
- unknown backend capability-query failure
- local backend capability-query stability
- Kubernetes backend capability-query stability
- HPC backend capability-query stability
- remote backend capability-query stability
- adapter metadata persistence in run outputs
- adapter metadata exclusion from graph identity
- registry selection determinism with multiple compatible adapters

Coverage anchors:
- `crates/bijux-dag-runtime/tests/adapter_registry_capability_contracts.rs`
- `crates/bijux-dag-runtime/tests/backend_capability_boundary_contracts.rs`
- `crates/bijux-dev-dag/tests/k8s_adapter_contracts.rs`
- `crates/bijux-dev-dag/tests/hpc_adapter_contracts.rs`
- `crates/bijux-dev-dag/tests/backend_equivalence_contracts.rs`

## 336-338 generated truth surfaces

- adapter coverage matrix:
  - `docs/reports/foundation/adapter_conformance_coverage_matrix.json`
- generated backend capability docs page:
  - `docs/reports/foundation/BACKEND_CAPABILITY_QUERY_REFERENCE.md`
- backend claim to evidence link map:
  - `docs/reports/foundation/BACKEND_CLAIMS_EVIDENCE_LINKS.md`

## 339 release gate for generated docs

- `crates/bijux-dev-dag/tests/backend_capability_docs_generation_contracts.rs`

## 340 backend conformance fast subset

- `configs/suites/backend_conformance_fast.json`
- `crates/bijux-dev-dag/tests/backend_conformance_fast_suite_contracts.rs`
