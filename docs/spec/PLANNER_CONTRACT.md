# Planner Contract

## Scope
Defines planner input/output, graph lowering semantics, plan identity, and runtime execution boundary.

## Planner input model
Planner consumes a parsed and validated DAG graph.

Boundary definitions:
- parsed graph: strict JSON parsed DAG structure.
- validated graph: parsed graph with schema and semantic validation passed.
- canonical graph: deterministic ordering form used for identity-sensitive operations.
- execution plan: runtime-facing lowered representation containing only execution-relevant fields.

## Lowering boundary
Planner performs explicit graph lowering from validated canonical graph to `ExecutionPlan`.

Erased user-facing fields in lowering:
- `graph.meta`
- `graph.inputs`
- `graph.nondeterminism_allowed`
- `node.tags`
- `node.group`
- `node.params`
- `node.resources`

Surviving execution fields:
- node id, kind, dependencies, retry, timeout, outputs
- lowered dependency edges
- deterministic execution ordering

Selection model:
Selection/filtering is applied after validation and before execution planning.

## Identity model
Planner emits:
- graph fingerprint: identity of canonical graph.
- planner fingerprint: identity of lowered execution plan.

Both are stable and deterministic for semantically identical inputs.

## Diagnostics
Planner diagnostics use stable IDs:
- `P4000`: planner generic failure.
- `P4013`: unsupported runtime node kind.
- `P4016`: execution no-op or annotation-only node warning.

## Runtime boundary
Runtime execution contract is plan-first: runtime execution is driven by lowered plan semantics, not author-facing graph metadata.

## Debug and schema surfaces
- Plan JSON schema: `configs/schema/execution_plan.schema.json`
- Debug dump command: `bijux-dev-dag dag plan-dump --graph <path>`

## Related tests
- `crates/bijux-dag-core/tests/planner_contract.rs`

## Versioning and change policy
Planner contract changes require synchronized updates to this doc, planner schema, and planner contract tests.
