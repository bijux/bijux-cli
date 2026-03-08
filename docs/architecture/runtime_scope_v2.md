# Runtime Scope v2

## Runtime Contraction Outcome (61-80)
- Runtime ownership is constrained to engine, scheduler, state machine, backend binding, cache, policy, replay, diagnostics.
- Speculative, productization, and distributed-future modules are explicitly marked `move` and blocked from scope creep by policy guardrails.
- Modeled platform and product surfaces are no longer advertised from the crate root; they are exposed only through `bijux_dag_runtime::simulated_platform`.
- Execution plan home is fixed to `runtime_core/planning/execution_plan.rs`.
- Planning bridge home is fixed to `runtime_core/planning/planner.rs`.
- Top-level runtime module freeze allows only approved root directories, `lib.rs`, and the explicit quarantine facade `simulated_platform.rs`.

## Classification Summary
- `backend`: 26 modules
- `core-runtime`: 37 modules
- `diagnostics`: 7 modules
- `policy`: 3 modules
- `replay`: 4 modules
- `security`: 7 modules
- `speculative`: 13 modules
- `support`: 29 modules
- `wrong-crate`: 1 modules

## Named Decisions (44-58)
- `geo_federation`: `move`
- `ha_scheduler`: `move`
- `federated_scheduling`: `move`
- `control_plane_api`: `move`
- `operations_governance`: `move`
- `adaptive_scheduler`: `move`
- `cost_optimization`: `move`
- `dataset_semantics`: `move`
- `formal_verification`: `keep`
- `ai_operator_assist`: `move`
- `workflow_product`: `move`
- `tenancy`: `move`
- `provenance_compliance`: `move`
- `supply_chain_trust`: `move`
- `execution_plan_home`: `runtime_core/planning/execution_plan.rs`
- `planner_bridge_home`: `runtime_core/planning/planner.rs`

## Remote, Container, Batch Maturity (58)
- `backend/runtime/batch_execution.rs`: `simulated-only`
- `backend/runtime/container_execution.rs`: `simulated-only`
- `backend/runtime/remote_execution_model.rs`: `future`
- `backend/runtime/remote_executor.rs`: `future`

## Hard Keep List (60)
- `artifacts/storage/path_authorization.rs`
- `artifacts/storage/recovery.rs`
- `artifacts/storage/store.rs`
- `artifacts/storage/trace.rs`
- `cache/key.rs`
- `cache/store.rs`
- `policy/evaluator.rs`
- `policy/trace.rs`
- `replay/diff.rs`
- `replay/explain.rs`
- `replay/verifier.rs`
- `runtime_core/execution/engine.rs`
- `runtime_core/execution/node_result.rs`
- `runtime_core/execution/run_context.rs`
- `runtime_core/execution/run_state.rs`
- `runtime_core/execution/scheduler.rs`
- `runtime_core/execution/state_machine.rs`
- `runtime_core/planning/execution_plan.rs`
- `runtime_core/planning/planner.rs`

## Full Module Inventory (41-43)
| Module | Classification | Decision | Notes |
| --- | --- | --- | --- |
| `simulated_platform.rs` | `support` | `keep` | explicit quarantine facade for modeled platform surfaces that are not part of the stable runtime root |
| `adapters/adapter.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/api.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/async_adapter.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/builtins/const_value.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/builtins/container.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/builtins/mod.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/builtins/shell.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/conformance.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/contract.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/external.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/mod.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/registry.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/runtime_registry.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `adapters/sdk.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `artifacts/manifest.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/mod.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/storage/path_authorization.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/storage/recovery.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/storage/semantic_lineage.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/storage/store.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/storage/trace.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/storage/upgrade_compatibility.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/verifier.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `artifacts/writer.rs` | `core-runtime` | `keep` | artifact write verify and storage semantics |
| `backend/capability.rs` | `support` | `keep` | runtime support module |
| `backend/contract.rs` | `support` | `keep` | runtime support module |
| `backend/distributed/coordination.rs` | `speculative` | `move` | distributed extension surface should be isolated from foundation runtime kernel |
| `backend/distributed/distributed.rs` | `speculative` | `move` | distributed extension surface should be isolated from foundation runtime kernel |
| `backend/distributed/distribution_readiness.rs` | `speculative` | `move` | distributed extension surface should be isolated from foundation runtime kernel |
| `backend/distributed/federated_scheduling.rs` | `speculative` | `move` | federated scheduling semantics should live outside foundation runtime core |
| `backend/distributed/geo_federation.rs` | `speculative` | `move` | future geo federation control plane capability should not expand foundation runtime |
| `backend/distributed/ha_scheduler.rs` | `speculative` | `move` | high-availability scheduler model belongs to future distributed execution scope |
| `backend/distributed/infrastructure.rs` | `speculative` | `move` | distributed extension surface should be isolated from foundation runtime kernel |
| `backend/fake.rs` | `support` | `keep` | runtime support module |
| `backend/local_process.rs` | `support` | `keep` | runtime support module |
| `backend/mod.rs` | `support` | `keep` | runtime support module |
| `backend/runtime/backend_cluster.rs` | `backend` | `keep` | backend capability and local execution integration surface |
| `backend/runtime/batch_execution.rs` | `backend` | `move` | batch execution support is future execution-mode boundary and should remain non-foundation |
| `backend/runtime/container_execution.rs` | `backend` | `move` | container execution support is future execution-mode boundary and should remain non-foundation |
| `backend/runtime/execution_backend.rs` | `backend` | `keep` | backend capability and local execution integration surface |
| `backend/runtime/local_executor.rs` | `backend` | `keep` | backend capability and local execution integration surface |
| `backend/runtime/remote_execution_model.rs` | `backend` | `move` | remote execution model is future-distributed boundary and should not define kernel scope |
| `backend/runtime/remote_executor.rs` | `backend` | `move` | remote executor is future-distributed boundary and should not define kernel scope |
| `backend/runtime/subprocess.rs` | `backend` | `keep` | backend capability and local execution integration surface |
| `builtins/const_adapter.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `builtins/container_adapter.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `builtins/mod.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `builtins/shell_adapter.rs` | `backend` | `keep` | adapter integration surface required for node execution |
| `cache/key.rs` | `core-runtime` | `keep` | cache identity proof and storage semantics |
| `cache/lineage.rs` | `core-runtime` | `keep` | cache identity proof and storage semantics |
| `cache/mod.rs` | `core-runtime` | `keep` | cache identity proof and storage semantics |
| `cache/proof.rs` | `core-runtime` | `keep` | cache identity proof and storage semantics |
| `cache/store.rs` | `core-runtime` | `keep` | cache identity proof and storage semantics |
| `diagnostics/events.rs` | `diagnostics` | `keep` | runtime diagnostics and eventing surface |
| `diagnostics/ids.rs` | `diagnostics` | `keep` | runtime diagnostics and eventing surface |
| `diagnostics/mod.rs` | `diagnostics` | `keep` | runtime diagnostics and eventing surface |
| `diagnostics/runtime/control_plane.rs` | `diagnostics` | `keep` | runtime diagnostics and eventing surface |
| `diagnostics/runtime/control_plane_api.rs` | `wrong-crate` | `move` | control-plane api surface belongs in app/dev control plane layer |
| `diagnostics/runtime/observability.rs` | `diagnostics` | `keep` | runtime diagnostics and eventing surface |
| `diagnostics/runtime/observability_deep.rs` | `diagnostics` | `keep` | runtime diagnostics and eventing surface |
| `diagnostics/runtime/operations_governance.rs` | `speculative` | `move` | operations governance scorecard logic is non-foundation runtime scope |
| `diagnostics/timeline.rs` | `diagnostics` | `keep` | runtime diagnostics and eventing surface |
| `error/classify.rs` | `core-runtime` | `keep` | runtime error model and classification |
| `error/codes.rs` | `core-runtime` | `keep` | runtime error model and classification |
| `error/mod.rs` | `core-runtime` | `keep` | runtime error model and classification |
| `internal/analysis/adaptive_scheduler.rs` | `speculative` | `move` | adaptive scheduler intelligence is beyond deterministic foundation scheduler scope |
| `internal/analysis/cost_optimization.rs` | `speculative` | `move` | cost optimization models are advisory platform concerns not core runtime semantics |
| `internal/analysis/dataset_semantics.rs` | `speculative` | `move` | dataset product semantics are higher-level than runtime execution kernel |
| `internal/clock.rs` | `support` | `keep` | internal runtime support module |
| `internal/control/api.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/control/clock.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/control/config.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/control/io.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/control/runtime.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/control/selectors.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/control/services.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/control/task_contract.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/control/task_types.rs` | `support` | `keep` | runtime control helpers and typed surfaces |
| `internal/ext/extension_catalog.rs` | `support` | `move` | extension and catalog support should be isolated from runtime kernel |
| `internal/ext/formal_verification.rs` | `support` | `keep` | verification helpers support invariants without changing runtime execution semantics |
| `internal/identity/auth_identity.rs` | `security` | `keep` | security and identity constraints used by runtime |
| `internal/identity/authz_policy.rs` | `security` | `keep` | security and identity constraints used by runtime |
| `internal/identity/provenance_compliance.rs` | `security` | `move` | provenance compliance policy should be isolated from runtime kernel surface |
| `internal/identity/secrets_security.rs` | `security` | `keep` | security and identity constraints used by runtime |
| `internal/identity/security_env.rs` | `security` | `keep` | security and identity constraints used by runtime |
| `internal/identity/supply_chain_trust.rs` | `security` | `move` | supply chain trust policy should be isolated from runtime kernel surface |
| `internal/identity/tenancy.rs` | `security` | `move` | tenancy policy should be isolated from core runtime kernel surface |
| `internal/io.rs` | `support` | `keep` | internal runtime support module |
| `internal/mod.rs` | `support` | `keep` | internal runtime support module |
| `internal/perf/performance_capacity.rs` | `support` | `move` | performance maturity reporting is governance support not kernel runtime |
| `internal/selectors.rs` | `support` | `keep` | internal runtime support module |
| `internal/testing/adapter_contract_tests.rs` | `support` | `keep` | runtime-internal tests and support fixtures |
| `internal/testing/invariants_tests.rs` | `support` | `keep` | runtime-internal tests and support fixtures |
| `internal/testing/runtime_boundary_tests.rs` | `support` | `keep` | runtime-internal tests and support fixtures |
| `internal/testing/runtime_policy_trace_tests.rs` | `support` | `keep` | runtime-internal tests and support fixtures |
| `internal/testing/state_machine_tests.rs` | `support` | `keep` | runtime-internal tests and support fixtures |
| `internal/testing/test_support.rs` | `support` | `keep` | runtime-internal tests and support fixtures |
| `internal/testing/tests_runtime.in.rs` | `support` | `keep` | runtime-internal tests and support fixtures |
| `internal/workflow/ai_operator_assist.rs` | `speculative` | `move` | ai operator assist is non-foundation workflow augmentation |
| `internal/workflow/workflow_product.rs` | `speculative` | `move` | workflow productization scorecards are out of runtime kernel scope |
| `policy/evaluator.rs` | `policy` | `keep` | policy evaluation in execution path |
| `policy/mod.rs` | `policy` | `keep` | policy evaluation in execution path |
| `policy/trace.rs` | `policy` | `keep` | policy evaluation in execution path |
| `replay/diff.rs` | `replay` | `keep` | replay diff verification behavior |
| `replay/explain.rs` | `replay` | `keep` | replay diff verification behavior |
| `replay/mod.rs` | `replay` | `keep` | replay diff verification behavior |
| `replay/verifier.rs` | `replay` | `keep` | replay diff verification behavior |
| `runtime_core/execution/context.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/execution/engine.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/execution/flow.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/execution/node_result.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/execution/run_context.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/execution/run_state.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/execution/scheduler.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/execution/scheduler_workload.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/execution/state_machine.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/governance/invariants.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/governance/sacred_execution.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/governance/semantics.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/mod.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/planning/execution_plan.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/planning/planner.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/planning/planner_analysis.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/state/mod.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/state/node_state.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
| `runtime_core/state/run_state.rs` | `core-runtime` | `keep` | core runtime execution and planning kernel surface |
