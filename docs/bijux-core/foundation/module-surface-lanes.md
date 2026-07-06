# Module Surface Lanes

Source contract: `contracts/foundation/module_surface_lanes.v1.json`.

This document defines how top-level `lib.rs` modules are classified in the workspace.

- `stable`: public module contract intended for normal crate consumers.
- `experimental`: public module contract allowed to evolve without the same compatibility guarantees.
- `simulated`: public module contract that models behavior not yet enforced as production capability.
- `private`: internal module contract, not exported through `pub mod` at crate root.

## Rules

- `bijux-dag-cli` is `binary-only` and must not expose a library target.
- For all library crates in the contract, internal modules default to `private`.
- Public module lanes are owned by the contract and enforced by `foundation_module_surface_contracts`.
- `stable` and `prelude` modules define the intentional import lanes for the DAG crates.
- `bijux-dag-runtime` keeps its module-level public surface intentionally narrow:
  `stable`, `prelude`, `experimental`, and `simulated_platform`.
- `experimental` modules are opt-in compatibility lanes behind crate features and must not expose transitional module names directly.
- `bijux-dag-runtime::simulated_platform` is the only `simulated` lane in the current Level-1 foundation set.
