# Boundary rules

## Forbidden crate dependencies

The following dependencies are forbidden:

- `bijux-dag-runtime -> bijux-dag-app`
- `bijux-dag-runtime -> bijux-dag-cli`
- `bijux-dag-core -> bijux-dag-runtime`
- `bijux-dev-dag -> bijux-dag-runtime`

## Rationale

- Prevent execution internals from leaking into app and CLI orchestration layers.
- Keep core independent of runtime policy and side-effect boundaries.
- Keep development control-plane tooling independent from runtime internals.

## Source of truth

Machine-enforced rules live in `configs/policy/dependency_rules.json`.
