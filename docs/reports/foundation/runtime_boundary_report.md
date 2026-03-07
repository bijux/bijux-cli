# Runtime Boundary Report

- tracked_modules: 126
- keep_modules: 103
- move_modules: 23

## Move Queue

- `backend/distributed/coordination.rs` (speculative): distributed extension surface should be isolated from foundation runtime kernel
- `backend/distributed/distributed.rs` (speculative): distributed extension surface should be isolated from foundation runtime kernel
- `backend/distributed/distribution_readiness.rs` (speculative): distributed extension surface should be isolated from foundation runtime kernel
- `backend/distributed/federated_scheduling.rs` (speculative): federated scheduling semantics should live outside foundation runtime core
- `backend/distributed/geo_federation.rs` (speculative): future geo federation control plane capability should not expand foundation runtime
- `backend/distributed/ha_scheduler.rs` (speculative): high-availability scheduler model belongs to future distributed execution scope
- `backend/distributed/infrastructure.rs` (speculative): distributed extension surface should be isolated from foundation runtime kernel
- `backend/runtime/batch_execution.rs` (backend): batch execution support is future execution-mode boundary and should remain non-foundation
- `backend/runtime/container_execution.rs` (backend): container execution support is future execution-mode boundary and should remain non-foundation
- `backend/runtime/remote_execution_model.rs` (backend): remote execution model is future-distributed boundary and should not define kernel scope
- `backend/runtime/remote_executor.rs` (backend): remote executor is future-distributed boundary and should not define kernel scope
- `diagnostics/runtime/control_plane_api.rs` (wrong-crate): control-plane api surface belongs in app/dev control plane layer
- `diagnostics/runtime/operations_governance.rs` (speculative): operations governance scorecard logic is non-foundation runtime scope
- `internal/analysis/adaptive_scheduler.rs` (speculative): adaptive scheduler intelligence is beyond deterministic foundation scheduler scope
- `internal/analysis/cost_optimization.rs` (speculative): cost optimization models are advisory platform concerns not core runtime semantics
- `internal/analysis/dataset_semantics.rs` (speculative): dataset product semantics are higher-level than runtime execution kernel
- `internal/ext/extension_catalog.rs` (support): extension and catalog support should be isolated from runtime kernel
- `internal/identity/provenance_compliance.rs` (security): provenance compliance policy should be isolated from runtime kernel surface
- `internal/identity/supply_chain_trust.rs` (security): supply chain trust policy should be isolated from runtime kernel surface
- `internal/identity/tenancy.rs` (security): tenancy policy should be isolated from core runtime kernel surface
- `internal/perf/performance_capacity.rs` (support): performance maturity reporting is governance support not kernel runtime
- `internal/workflow/ai_operator_assist.rs` (speculative): ai operator assist is non-foundation workflow augmentation
- `internal/workflow/workflow_product.rs` (speculative): workflow productization scorecards are out of runtime kernel scope
