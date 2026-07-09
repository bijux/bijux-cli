# Runtime Non-Kernel Modules

Generated from `configs/dag/policy/runtime_scope_v2.json`.

Performance-related classifications on this page must stay backed by `bijux-dev-dag performance-evidence-report` and `evidence/perf/metadata.json`.

## Runtime modules outside kernel ownership

- `adapters/adapter.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/api.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/async_adapter.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/builtins/const_value.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/builtins/container.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/builtins/mod.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/builtins/shell.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/conformance.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/contract.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/external.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/mod.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/registry.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/runtime_registry.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `adapters/sdk.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `backend/capability.rs` (support, decision `keep`): runtime support module
- `backend/contract.rs` (support, decision `keep`): runtime support module
- `backend/distributed/coordination.rs` (speculative, decision `move`): distributed extension surface should be isolated from foundation runtime kernel
- `backend/distributed/distributed.rs` (speculative, decision `move`): distributed extension surface should be isolated from foundation runtime kernel
- `backend/distributed/distribution_readiness.rs` (speculative, decision `move`): distributed extension surface should be isolated from foundation runtime kernel
- `backend/distributed/federated_scheduling.rs` (speculative, decision `move`): federated scheduling semantics should live outside foundation runtime core
- `backend/distributed/geo_federation.rs` (speculative, decision `move`): unreleased geo federation control plane capability should not expand foundation runtime
- `backend/distributed/ha_scheduler.rs` (speculative, decision `move`): high-availability scheduler model belongs to unreleased distributed execution scope
- `backend/distributed/infrastructure.rs` (speculative, decision `move`): distributed extension surface should be isolated from foundation runtime kernel
- `backend/fake.rs` (support, decision `keep`): runtime support module
- `backend/local_process.rs` (support, decision `keep`): runtime support module
- `backend/mod.rs` (support, decision `keep`): runtime support module
- `backend/runtime/backend_cluster.rs` (backend, decision `keep`): backend capability and local execution integration surface
- `backend/runtime/batch_execution.rs` (backend, decision `move`): batch execution support is modeled execution-mode boundary and should remain non-foundation
- `backend/runtime/container_execution.rs` (backend, decision `move`): container execution support is modeled execution-mode boundary and should remain non-foundation
- `backend/runtime/execution_backend.rs` (backend, decision `keep`): backend capability and local execution integration surface
- `backend/runtime/local_executor.rs` (backend, decision `keep`): backend capability and local execution integration surface
- `backend/runtime/remote_execution_model.rs` (backend, decision `move`): remote execution model is unreleased distributed boundary and should not define kernel scope
- `backend/runtime/remote_executor.rs` (backend, decision `move`): remote executor is unreleased distributed boundary and should not define kernel scope
- `backend/runtime/subprocess.rs` (backend, decision `keep`): backend capability and local execution integration surface
- `builtins/const_adapter.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `builtins/container_adapter.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `builtins/mod.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `builtins/shell_adapter.rs` (backend, decision `keep`): adapter integration surface required for node execution
- `diagnostics/events.rs` (diagnostics, decision `keep`): runtime diagnostics and eventing surface
- `diagnostics/ids.rs` (diagnostics, decision `keep`): runtime diagnostics and eventing surface
- `diagnostics/mod.rs` (diagnostics, decision `keep`): runtime diagnostics and eventing surface
- `diagnostics/runtime/control_plane.rs` (diagnostics, decision `keep`): runtime diagnostics and eventing surface
- `diagnostics/runtime/control_plane_api.rs` (wrong-crate, decision `move`): control-plane api surface belongs in app/dev control plane layer
- `diagnostics/runtime/observability.rs` (diagnostics, decision `keep`): runtime diagnostics and eventing surface
- `diagnostics/runtime/observability_deep.rs` (diagnostics, decision `keep`): runtime diagnostics and eventing surface
- `diagnostics/runtime/operations_governance.rs` (speculative, decision `move`): operations governance scorecard logic is non-foundation runtime scope
- `diagnostics/timeline.rs` (diagnostics, decision `keep`): runtime diagnostics and eventing surface
- `error/classify.rs` (core-runtime, decision `keep`): runtime error model and classification
- `error/codes.rs` (core-runtime, decision `keep`): runtime error model and classification
- `error/mod.rs` (core-runtime, decision `keep`): runtime error model and classification
- `internal/analysis/adaptive_scheduler.rs` (speculative, decision `move`): adaptive scheduler intelligence is beyond deterministic foundation scheduler scope
- `internal/analysis/cost_optimization.rs` (speculative, decision `move`): cost optimization models are advisory platform concerns not core runtime semantics
- `internal/analysis/dataset_semantics.rs` (speculative, decision `move`): dataset product semantics are higher-level than runtime execution kernel
- `internal/clock.rs` (support, decision `keep`): internal runtime support module
- `internal/control/api.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/control/clock.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/control/config.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/control/io.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/control/runtime.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/control/selectors.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/control/services.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/control/node_execution_contract.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/control/node_execution_types.rs` (support, decision `keep`): runtime control helpers and typed surfaces
- `internal/ext/extension_catalog.rs` (support, decision `move`): extension and catalog support should be isolated from runtime kernel
- `internal/ext/formal_verification.rs` (support, decision `keep`): verification helpers support invariants without changing runtime execution semantics
- `internal/identity/auth_identity.rs` (security, decision `keep`): security and identity constraints used by runtime
- `internal/identity/authz_policy.rs` (security, decision `keep`): security and identity constraints used by runtime
- `internal/identity/provenance_compliance.rs` (security, decision `move`): provenance compliance policy should be isolated from runtime kernel surface
- `internal/identity/secrets_security.rs` (security, decision `keep`): security and identity constraints used by runtime
- `internal/identity/security_env.rs` (security, decision `keep`): security and identity constraints used by runtime
- `internal/identity/supply_chain_trust.rs` (security, decision `move`): supply chain trust policy should be isolated from runtime kernel surface
- `internal/identity/tenancy.rs` (security, decision `move`): tenancy policy should be isolated from core runtime kernel surface
- `internal/io.rs` (support, decision `keep`): internal runtime support module
- `internal/mod.rs` (support, decision `keep`): internal runtime support module
- `internal/perf/performance_capacity.rs` (support, decision `move`): performance maturity reporting is governance support not kernel runtime
- `internal/selectors.rs` (support, decision `keep`): internal runtime support module
- `internal/testing/adapter_contract_tests.rs` (support, decision `keep`): runtime-internal tests and support fixtures
- `internal/testing/invariants_tests.rs` (support, decision `keep`): runtime-internal tests and support fixtures
- `internal/testing/runtime_boundary_tests.rs` (support, decision `keep`): runtime-internal tests and support fixtures
- `internal/testing/runtime_policy_trace_tests.rs` (support, decision `keep`): runtime-internal tests and support fixtures
- `internal/testing/state_machine_tests.rs` (support, decision `keep`): runtime-internal tests and support fixtures
- `internal/testing/test_support.rs` (support, decision `keep`): runtime-internal tests and support fixtures
- `internal/testing/tests_runtime.in.rs` (support, decision `keep`): runtime-internal tests and support fixtures
- `internal/workflow/ai_operator_assist.rs` (speculative, decision `move`): ai operator assist is non-foundation workflow augmentation
- `internal/workflow/workflow_product.rs` (speculative, decision `move`): workflow productization scorecards are out of runtime kernel scope
- `simulated_platform.rs` (support, decision `keep`): explicit quarantine facade for modeled platform surfaces that are not part of the stable runtime root

Total: `86` modules.
