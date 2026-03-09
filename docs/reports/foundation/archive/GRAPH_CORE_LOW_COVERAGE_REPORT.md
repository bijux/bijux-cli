# Graph Core Low Coverage Report

Generated: 2026-03-08

Scope:
- `crates/bijux-dag-core/src/graph/canonical.rs`
- `crates/bijux-dag-core/src/graph/edge.rs`
- `crates/bijux-dag-core/src/graph/topology.rs`
- `crates/bijux-dag-core/src/pipeline/validate.rs`
- `crates/bijux-dag-core/src/pipeline/resolve.rs`
- `crates/bijux-dag-core/src/planner/planner.rs`

Direct-test coverage status:
- `graph_pipeline_planner_expansion_contracts.rs` now exercises canonical minimal/maximal fixtures, edge legality checks, disconnected and fan-in/fan-out topology, validation/resolve behavior, and planner imported/selective replay flows.
- `direct_module_entrypoints_contracts.rs`, `planner_contract.rs`, and `planner_validation_remaining_contracts.rs` remain part of the direct entrypoint verification set.

Current finding for this scoped set:
- No intentionally weakly-covered file remains in this scoped graph/pipeline/planner slice.
