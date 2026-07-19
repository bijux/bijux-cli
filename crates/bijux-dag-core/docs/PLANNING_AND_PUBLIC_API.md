# Planning And Public API

Planner lowering converts a resolved, valid graph into an `ExecutionPlan` that
runtime can execute without interpreting the authored graph again.

## Planner Boundary

`lower_graph_to_execution_plan` produces planned nodes, edges, branch
contracts, diagnostics, and identity under `PlanOptions`. Planning must be
deterministic, independent of adapter availability, explicit about unsupported
node kinds, aligned with topology and triggers, and sufficient for execution.

Planner errors map through owned conversions. Callers must not classify planner
messages by string matching.

## Compile Helpers

- `compile_graph` is the normal compile path.
- `compile_graph_strict` enforces strict compatibility.
- `compile_graph_with_defaults` accepts explicit graph defaults.
- `compile_graph_contract` returns contract-oriented compile data.

Use the narrowest entrypoint matching the caller's input. Do not wrap these
helpers with a second validator or planner.

## Stable Surface

`bijux_dag_core::stable` is the long-lived compatibility lane. It exposes the
primary model, compile, canonicalization, composition, validation, planning,
diagnostics, and version contracts.

`prelude` groups common imports but does not widen stability. Crate-root
compatibility re-exports remain callable and hidden from primary docs; new
downstream code should prefer `stable` or focused imports.

The `experimental-public-api` feature exposes research contracts outside the
stable promise. Workspace usage alone does not justify promotion. Stable
promotion requires durable consumers, ownership, documentation, compatibility
tests, and release review.

## Runtime Handoff

Core hands runtime canonical node and dependency identity, resolved params,
branch and trigger behavior, declared resources, retries, effects, outputs,
and refusal diagnostics. Runtime owns scheduling, adapters, concrete paths,
attempts, cache lookup, and retained evidence.

## Verification

```bash
cargo test --locked -p bijux-dag-core --test planner_contract
cargo test --locked -p bijux-dag-core --test planner_fixture_contracts
cargo test --locked -p bijux-dag-core --test prelude_contract
cargo test --locked -p bijux-dag-core --test direct_module_entrypoints_contracts
```

Planner scale budgets and edge-case contracts accompany changes to lowering
complexity or supported semantics.
