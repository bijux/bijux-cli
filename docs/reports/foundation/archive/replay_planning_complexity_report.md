# Replay Planning Complexity Report

Generated: 2026-03-08

## Scope

Replay planning complexity is evaluated for:

- imported-run replay planning
- selective replay closure construction
- replay proof and dry-run planning paths

## Complexity surfaces

- planner lowering and selection:
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs`
- CLI replay route and imported-run handling:
  - `crates/bijux-dag-app/tests/replay_lineage_planning_contract.rs`

## Current posture

- replay plan construction remains deterministic for identical graph semantics
- selective replay closure remains bounded to selected dependency closure
- imported-run replay planning remains supported through manifest lineage continuity
