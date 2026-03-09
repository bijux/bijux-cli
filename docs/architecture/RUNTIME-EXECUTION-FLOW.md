# Runtime execution flow

Execution path:
1. Planner builds deterministic execution plan from graph + runtime config.
2. Executor materializes inputs, computes fingerprints, and resolves adapter dispatch.
3. Adapter execution produces outputs and status.
4. Trace writer persists per-node trace, attempt events, and resolved params.
5. Artifact persistence writes manifest, outputs indexes, provenance, and run summary.
6. Run state advances to terminal status based on canonical node accounting.

Centralized sacred hooks:
- `sacred_execution::resolve_dependencies`
- `sacred_execution::ready_queue_from_dependencies`
- `sacred_execution::run_materialize_inputs`
- `sacred_execution::run_cache_lookup`
- `sacred_execution::run_retry_logic`
- `sacred_execution::run_write_trace`
- `sacred_execution::run_cache_write`

Data-flow contract:
- Planner and policy evaluation are deterministic for identical input state.
- Trace writing is append-only per node attempt and final status.
- Artifact persistence is schema-bound by run manifest and trace schemas.

Effect boundaries:
- run-scoped dependencies are carried by `execution_context::ExecutionContext`.
- node-scoped dependencies are carried by `execution_context::NodeExecutionContext`.
- engine code must not bypass sacred cache/trace/retry hooks.
