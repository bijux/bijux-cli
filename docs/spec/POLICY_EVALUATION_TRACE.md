# Policy Evaluation Trace

## Scope
Defines debug-mode policy trace expectations.

## Contract
- Debug output includes policy rule decisions (`allow`/`deny`) with rule labels.
- Default output excludes detailed policy trace internals.
- Trace format is machine-readable JSON.

## Related tests
- `crates/bijux-dag-app/tests/policy_mode_contract.rs`

## Versioning and change policy
Trace field additions are additive. Field removals require contract update and snapshot refresh.
