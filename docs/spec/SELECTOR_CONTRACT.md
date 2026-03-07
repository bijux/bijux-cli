# Selector Contract

## Scope
Defines selection and exclusion semantics for node execution targeting.

## Invariants
- Selection filtering is deterministic.
- Include and exclude interaction rules are explicit and stable.
- Invalid selector references fail validation before execution.

## Related tests
- `evidence/battle/workflows/selection/*`
- `crates/bijux-dag-runtime/tests/selector_filtering_contract.rs`

## Related schemas
- `configs/schema/dag.schema.json`

## Versioning and change policy
Selector semantic changes require compatibility review and updated integration coverage.
