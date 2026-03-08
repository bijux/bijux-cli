# Runtime Quarantined Surface Owner Mapping

generated_from: `configs/policy/runtime_broad_surface_ownership.json`

| module | decision | owner_repo |
| --- | --- | --- |
| backend/distributed/coordination.rs | quarantine | bijux-control-plane |
| backend/distributed/distributed.rs | quarantine | bijux-runtime-extensions |
| backend/distributed/distribution_readiness.rs | quarantine | bijux-dev-dag |
| backend/distributed/federated_scheduling.rs | quarantine | bijux-control-plane |
| backend/distributed/geo_federation.rs | quarantine | bijux-control-plane |
| backend/distributed/ha_scheduler.rs | quarantine | bijux-control-plane |
| diagnostics/runtime/control_plane.rs | quarantine | bijux-control-plane |
| diagnostics/runtime/control_plane_api.rs | quarantine | bijux-control-plane |
| internal/analysis/adaptive_scheduler.rs | quarantine | bijux-runtime-extensions |
| internal/analysis/cost_optimization.rs | quarantine | bijux-runtime-extensions |
| internal/analysis/dataset_semantics.rs | quarantine | bijux-runtime-extensions |
| internal/ext/extension_catalog.rs | quarantine | bijux-plugin-sdk |
| internal/identity/auth_identity.rs | quarantine | bijux-control-plane |
| internal/identity/authz_policy.rs | quarantine | bijux-control-plane |
| internal/identity/tenancy.rs | quarantine | bijux-control-plane |
| internal/identity/provenance_compliance.rs | quarantine | bijux-dev-dag |
| internal/workflow/ai_operator_assist.rs | quarantine | bijux-operator-assist |
| internal/workflow/workflow_product.rs | quarantine | bijux-workflow-product |
