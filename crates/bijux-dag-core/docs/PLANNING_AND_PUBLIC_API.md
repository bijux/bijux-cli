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

## Lowering Pipeline

```mermaid
flowchart LR
    graph["Resolved valid graph"]
    options["Explicit PlanOptions"]
    lower["Deterministic lowering"]
    diagnostics["Typed diagnostics"]
    plan["ExecutionPlan"]
    runtime["Runtime handoff"]
    refusal["Planner refusal"]

    graph --> lower
    options --> lower
    lower --> diagnostics
    lower -->|supported semantics| plan --> runtime
    lower -->|unsupported or inconsistent| refusal
```

Planning is pure with respect to execution infrastructure. Backend discovery,
filesystem allocation, process creation, cache lookup, and retained evidence
must not influence the plan. Those effects belong after the handoff.

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

## API Selection

| Consumer need | Entry surface | Stability obligation |
| --- | --- | --- |
| ordinary graph compilation | `compile_graph` | preserve stable input, diagnostics, and plan behavior |
| strict compatibility enforcement | `compile_graph_strict` | fail when compatibility requirements are not met |
| caller-owned defaults | `compile_graph_with_defaults` | make defaults explicit and test their identity effect |
| contract inspection | `compile_graph_contract` | preserve contract-oriented shape and classifications |
| broad downstream integration | `bijux_dag_core::stable` | covered by public compatibility review |
| repository research | `experimental-public-api` | no stable promise; callers must opt in deliberately |

Crate-root compatibility re-exports are not the preferred discovery surface.
Do not add a new re-export as a shortcut around module ownership or stability
review.

## Runtime Handoff

Core hands runtime canonical node and dependency identity, resolved params,
branch and trigger behavior, declared resources, retries, effects, outputs,
and refusal diagnostics. Runtime owns scheduling, adapters, concrete paths,
attempts, cache lookup, and retained evidence.

The handoff must be sufficient for runtime to execute without reopening the
authored graph. If runtime needs to infer a graph rule, planning has leaked an
authority or the plan contract is incomplete.

## Verification

```bash
cargo test --locked -p bijux-dag-core --test planner_contract
cargo test --locked -p bijux-dag-core --test planner_fixture_contracts
cargo test --locked -p bijux-dag-core --test prelude_contract
cargo test --locked -p bijux-dag-core --test direct_module_entrypoints_contracts
```

Planner scale budgets and edge-case contracts accompany changes to lowering
complexity or supported semantics.
