# Crate ownership and domain authority

## Domain map

- `bijux-dag-core`: model
- `bijux-dag-artifacts`: artifacts
- `bijux-dag-runtime`: execution
- `bijux-dag-app`: app orchestration
- `bijux-dag-cli`: CLI
- `bijux-dag-testkit`: repo governance
- `bijux-dev-dag`: repo governance

## Public module contract

The enforceable contract is `configs/policy/crate_ownership.json`.

`bijux-dev-dag` validates that each crate exports only declared public modules.
