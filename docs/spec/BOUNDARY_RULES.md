# Boundary rules

## Forbidden crate dependencies

The following dependencies are forbidden:

- `bijux-dag-runtime -> bijux-dag-app`
- `bijux-dag-runtime -> bijux-dag-cli`
- `bijux-dag-core -> bijux-dag-runtime`
- `bijux-dev-dag -> bijux-dag-runtime`

Additional policy boundaries:

- `bijux-dag-cli` must not depend directly on runtime internals (`bijux-dag-runtime`) or core semantics (`bijux-dag-core`).
- No crate may depend on another crate only to reuse formatting or JSON rendering helpers.

## Rationale

- Prevent execution internals from leaking into app and CLI orchestration layers.
- Keep core independent of runtime policy and side-effect boundaries.
- Keep development control-plane tooling independent from runtime internals.
- Keep the binary CLI as wiring-only.
- Keep display and rendering helpers local to their owning crate unless promoted to a neutral utility crate.

## Source of truth

Machine-enforced rules live in `configs/policy/dependency_rules.json`.
