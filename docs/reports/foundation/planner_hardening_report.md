# Planner hardening report

## Scope

Tracks planner-lowering authority, schema alignment, and trust-property binding.

## Current guarantees

- planner authority is `docs/spec/PLANNER_CONTRACT.md`
- canonical planner types are `ExecutionPlan`, `PlannedNode`, and lowered dependency edges
- runtime planner bridge delegates lowering to core planner boundary
- plan dump output is checked against schema-required fields from `configs/schema/execution_plan.schema.json`
- battle trust policy includes `tp_plan_truth`

## Gate linkage

- repo suite guard: `planner-alignment`
- test suites:
  - `crates/bijux-dag-core/tests/planner_contract.rs`
  - `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`
  - `crates/bijux-dev-dag/tests/planner_hardening_contracts.rs`
