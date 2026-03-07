# Trace Contract

## Scope
Defines trace event ordering, required fields, optional fields, and compatibility constraints.

## Invariants
- Event ordering per node is deterministic for equivalent runs.
- Required fields are always present for persisted events.
- Optional fields are additive and must not break consumers.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`

## Related schemas
- `configs/schema/node-trace.schema.json`

## Versioning and change policy
Breaking event-shape changes require schema versioning and migration plan.
