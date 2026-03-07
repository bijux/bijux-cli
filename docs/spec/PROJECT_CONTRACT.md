# Project Contract

## Scope
Defines product-level goals, non-goals, stability, and compatibility constraints for the DAG engine.

## Goals
- Provide a strict, minimal DAG IR.
- Ensure deterministic execution order.
- Produce reproducible run artifacts.

## Non-Goals
- Distributed execution.
- Dynamic graph mutation at runtime.
- Implicit network access.

## Compatibility
- Spec versions live in `spec/`.
- Breaking changes require a new version file.

## Stability
- JSON parsing is strict (`deny_unknown_fields`).
- Canonical output uses stable ordering.

## Related tests
- `crates/bijux-dag-core/tests/contract_stability.rs`
- `crates/bijux-dag-core/tests/canonicalization_ordering.rs`

## Versioning and change policy
Project-level contract changes are additive by default; breaking scope or compatibility changes require explicit contract version note and linked evidence updates.
