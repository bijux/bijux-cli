# Kernel Owned Runtime Modules

Generated from `configs/dag/policy/runtime_scope_v2.json`.

Performance-related classifications on this page must stay backed by `bijux-dev-dag performance-evidence-report` and `evidence/perf/metadata.json`.

## Runtime kernel-owned module set

- `artifacts/manifest.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/mod.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/storage/path_authorization.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/storage/recovery.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/storage/semantic_lineage.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/storage/store.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/storage/trace.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/storage/upgrade_compatibility.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/verifier.rs` (core-runtime): artifact write verify and storage semantics
- `artifacts/writer.rs` (core-runtime): artifact write verify and storage semantics
- `cache/key.rs` (core-runtime): cache identity proof and storage semantics
- `cache/lineage.rs` (core-runtime): cache identity proof and storage semantics
- `cache/mod.rs` (core-runtime): cache identity proof and storage semantics
- `cache/proof.rs` (core-runtime): cache identity proof and storage semantics
- `cache/store.rs` (core-runtime): cache identity proof and storage semantics
- `policy/evaluator.rs` (policy): policy evaluation in execution path
- `policy/mod.rs` (policy): policy evaluation in execution path
- `policy/trace.rs` (policy): policy evaluation in execution path
- `replay/diff.rs` (replay): replay diff verification behavior
- `replay/explain.rs` (replay): replay diff verification behavior
- `replay/mod.rs` (replay): replay diff verification behavior
- `replay/verifier.rs` (replay): replay diff verification behavior
- `runtime_core/execution/context.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/engine.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/engine_dispatch.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/engine_finalize.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/engine_metrics.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/engine_observe.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/engine_record.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/flow.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/node_result.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/run_context.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/run_state.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/scheduler.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/scheduler_workload.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/execution/state_machine.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/governance/invariants.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/governance/sacred_execution.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/governance/semantics.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/mod.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/planning/execution_plan.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/planning/planner.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/planning/planner_analysis.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/state/mod.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/state/node_state.rs` (core-runtime): core runtime execution and planning kernel surface
- `runtime_core/state/run_state.rs` (core-runtime): core runtime execution and planning kernel surface

Total: `46` modules.
