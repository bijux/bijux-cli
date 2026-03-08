# Graph Fixture Inventory Report

- Purpose: Deterministic DAG shape, schema, and canonicalization contract coverage.
- Owner suite: planner_fixture_contracts
- Owner crate: bijux-dag-core

| Fixture path | Owner suite | Owner crate |
| --- | --- | --- |
| `configs/schema/fixtures/compat/negative/unsupported_future_graph.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/compat/negative/unsupported_v0_0_graph.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/compat/positive/v0_1_fanout_graph.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/compat/positive/v0_1_legacy_graph.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/negative/future-required-behavior.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/negative/invalid-enum-container-engine.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/negative/invalid-output-path.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/negative/malformed-ref.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/negative/unknown-field.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/positive/diamond.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/positive/disconnected-groups.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/positive/empty-graph.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/positive/fan-in.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/positive/fan-out.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.1/positive/isolated-node.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `configs/schema/fixtures/v0.2-draft/positive/minimal_empty_graph.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/conditional_branch_join.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/conflicting_retry_timeout_policy.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/cycle.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/dangling.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/dup_id.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/duplicate_outputs_per_node.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/env_allowlist_no_env.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/forward_ref.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_bytes/raw_simple.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_bytes/raw_with_defaults.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_diff/raw_unsorted_env.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_diff/raw_unsorted_nodes.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/deep_dependency_tree.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/invalid_close/invalid_spec_alias_typo.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/invalid_close/path_traversal_near_valid.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/large_fan_in.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/large_fan_out.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/graph_identity/mixed_shell_container.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/illegal_id.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/illegal_output_path_traversal.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/invalid_env_reference.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/invalid_ref.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/invalid_resource_declaration.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/missing_effects.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/orphan.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/output_collision.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/partitioned_map_reduce.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/retry_nondet.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/shell_no_fs.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/template_composition.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/unknown_node_output.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/unreachable.json` | `planner_fixture_contracts` | `bijux-dag-core` |
| `crates/bijux-dag-core/tests/fixtures/unsupported_node_settings.json` | `planner_fixture_contracts` | `bijux-dag-core` |

Total fixtures: 49
