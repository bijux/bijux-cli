# Runtime Module Triage

This document is retained for continuity. The active runtime scope authority is:
- `docs/reports/foundation/archive/runtime_scope_v2.md`
- `configs/policy/runtime_scope_v2.json`
- `configs/policy/runtime_module_freeze.json`

## Classification rubric
- core: required for deterministic execution semantics
- support: supporting contract or policy logic that can remain internal
- speculative: roadmap-heavy or future-facing modeling that should not shape core runtime APIs
- misplaced: belongs in other crates or docs as boundary definitions

## Runtime module inventory and classification

| Module | Classification | Notes |
| --- | --- | --- |
| adapter.rs | core | adapter execution boundary |
| engine.rs | core | orchestration flow |
| scheduler.rs | core | readiness and dispatch |
| run_state.rs | core | run-level state semantics |
| state_machine.rs | core | state transitions |
| execution_backend.rs | core | backend binding and lifecycle |
| node_result.rs | core | canonical node result model |
| run_context.rs | core | execution context |
| policy.rs | core | execution policy controls |
| cache.rs | core | cache behavior in execution path |
| store.rs | core | storage access boundary |
| planner.rs | core | graph lowering to executable plan |
| trace.rs | core | execution trace model |
| subprocess.rs | core | local process execution surface |
| selectors.rs | core | graph selection semantics |
| config.rs | support | runtime config parsing |
| invariants.rs | support | invariant checks and registry |
| recovery.rs | support | restart/repair checks |
| security_env.rs | support | environment shaping |
| path_authorization.rs | support | path guardrails |
| registry.rs | support | adapter registry internals |
| io.rs | support | fs abstraction |
| builtins.rs | support | built-in adapter implementations |
| planner_analysis.rs | speculative | advanced planner heuristics |
| adaptive_scheduler.rs | speculative | adaptive tuning model |
| ai_operator_assist.rs | speculative | assistive analysis model |
| distribution_readiness.rs | speculative | product strategy modeling |
| workflow_product.rs | speculative | product-level workflow modeling |
| geo_federation.rs | speculative | future geo/federation semantics |
| federated_scheduling.rs | speculative | future federated scheduler semantics |
| scheduler_workload.rs | speculative | workload scheduling extensions |
| observability_deep.rs | speculative | deep observability modeling |
| semantic_lineage.rs | speculative | advanced lineage modeling |
| dataset_semantics.rs | speculative | dataset product semantics |
| performance_capacity.rs | support | benchmark support models |
| operations_governance.rs | support | operational policy support |
| control_plane.rs | misplaced | control-plane concerns should stay in dev/app surfaces |
| control_plane_api.rs | misplaced | api/control-plane concerns outside core runtime |
| api.rs | misplaced | should remain thin compatibility surface only |
| remote_executor.rs | support | remote executor placeholder boundary |
| remote_execution_model.rs | support | remote model contracts |
| container_execution.rs | support | container contract mapping |
| batch_execution.rs | support | batch model boundaries |
| backend_cluster.rs | support | backend capability mappings |
| infrastructure.rs | support | backend requirement matching |
| external_adapter.rs | support | external adapter descriptors |
| adapter_sdk.rs | support | adapter plugin contract |
| extension_catalog.rs | support | extension contracts |
| auth_identity.rs | support | identity policy models |
| authz_policy.rs | support | authorization policy models |
| secrets_security.rs | support | secret redaction boundary |
| supply_chain_trust.rs | support | provenance/trust models |
| tenancy.rs | support | tenant policy models |
| cost_optimization.rs | speculative | optimization model |
| formal_verification.rs | support | invariant and proof model utilities |
| clock.rs | support | clock abstraction |
| coordination.rs | support | runtime coordination model |
| distributed.rs | support | distributed simulation model |
| execution_plan.rs | core | typed executable plan |
| task_contract.rs | support | task contract typing |
| task_types.rs | support | task type compatibility |
| upgrade_compatibility.rs | support | compatibility support logic |

## Sacred runtime modules
- `engine`
- `scheduler`
- `run_state`
- `state_machine`
- `execution_backend`
- `node_result`
- `run_context`
- `policy`
- `cache`
- `store`
- `planner`
- `trace`

## Immediate actions
- Freeze new runtime module creation behind explicit governance checks.
- Keep speculative modules internal and non-normative.
- Keep misplaced modules documented as boundary debt; do not expand their scope inside runtime.
