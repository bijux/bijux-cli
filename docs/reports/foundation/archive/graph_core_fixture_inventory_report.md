# Graph Core Fixture Inventory Report

Generated: 2026-03-08

Canonicalization fixtures:
- `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_bytes/raw_simple.json`
- `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_bytes/raw_with_defaults.json`
- `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_diff/raw_unsorted_env.json`
- `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_diff/raw_unsorted_nodes.json`

Topology fixtures:
- `crates/bijux-dag-core/tests/snapshots/fan_in.dag.json`
- `crates/bijux-dag-core/tests/snapshots/fan_out.dag.json`
- `crates/bijux-dag-core/tests/snapshots/diamond.dag.json`
- `crates/bijux-dag-core/tests/snapshots/isolated_groups.dag.json`

Validation and resolve fixtures:
- `crates/bijux-dag-core/tests/fixtures/output_collision.json`
- `crates/bijux-dag-core/tests/fixtures/invalid_env_reference.json`
- `crates/bijux-dag-core/tests/fixtures/unknown_node_output.json`
- `crates/bijux-dag-core/tests/fixtures/invalid_ref.json`

Planner fixtures:
- `crates/bijux-dag-core/tests/snapshots/imported_bundle_replay.dag.json`
- `crates/bijux-dag-core/tests/snapshots/selective_replay.dag.json`
- `crates/bijux-dag-core/tests/snapshots/replay_oriented.dag.json`

Inventory note:
- All fixtures above are wired through active direct tests and/or planner validation contracts.
