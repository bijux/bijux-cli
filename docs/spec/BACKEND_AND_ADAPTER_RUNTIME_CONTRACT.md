# Backend and adapter runtime contract

**What this spec is not**: benchmark strategy, scheduler tuning, or high-level governance philosophy.

## Scope

This contract is the canonical source for:

- runtime backend lifecycle and protocol expectations
- adapter identity, capability surfaces, and execution behavior
- backend equivalence and portability semantics
- backend placement and conformance boundaries

## Contract boundaries

- `BACKEND_CONTRACT`, `ADAPTER_CONTRACT`, and adapter interface specs are normative together.
- Backend-specific contracts for `k8s` and `hpc` are appendices to this cluster.
- Adapter semantics must not redefine run/graph meaning.

## Core requirements

- Backend lifecycle is explicit and deterministic in `prepare`, `launch`, `observe`, `finalize`, `cleanup`.
- Adapter identity and capabilities are stable and versioned.
- Conformance requires deterministic execution, error normalization, and cleanup guarantees.
- Cross-backend replay and equivalence behavior must be explicitly classified (`equivalent`, `fidelity-preserving`, `downgraded`).

## Evidence and implementation links

- Runtime enforcement: `crates/bijux-dag-runtime`
- Evidence and governance: `crates/bijux-dev-dag` backend conformance suites
- Canonical schemas/fixtures in backend and adapter test registries.

## Canonical appendices

- [backend contract](./appendices/backend/BACKEND_CONTRACT.md)
- [adapter contract](./appendices/backend/ADAPTER_CONTRACT.md)
- [adapter interface](./appendices/backend/ADAPTER_INTERFACE_SPEC_v0.1.md)
- [backend protocol stability](./appendices/backend/BACKEND_PROTOCOL_STABILITY_CONTRACT.md)
- [adapter runtime contract](./appendices/backend/ADAPTER_RUNTIME_CONTRACT_v0.1.md)
- [adapter placement and boundaries](./appendices/backend/ADAPTER_PLACEMENT.md)
- [backend execution maturity](./appendices/backend/BACKEND_EXECUTION_MATURITY.md)
- [backend equivalence](./appendices/backend/BACKEND_EQUIVALENCE_CONTRACT.md)
- [backend meaning boundary doctrine](./appendices/backend/BACKEND_MEANING_BOUNDARY_DOCTRINE.md)
- [kubernetes adapter](./appendices/backend/K8S_ADAPTER_CONTRACT.md)
- [hpc adapter](./appendices/backend/HPC_ADAPTER_CONTRACT.md)
- [atlas execution](./appendices/backend/ATLAS_EXECUTION_CONTRACT.md)
