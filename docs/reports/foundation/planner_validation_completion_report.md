# Planner Validation Completion Report (Tasks 61-80)

This report maps tasks 61-80 to executable coverage and fixtures.

## Validation fixtures (61-69)

- 61 ambiguous dependency declarations:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`validation_rejects_ambiguous_dependency_declarations`)
- 62 conflicting retry/timeout policies:
  - `crates/bijux-dag-core/tests/fixtures/conflicting_retry_timeout_policy.json`
  - `crates/bijux-dag-core/tests/validation_fixtures.rs` (`fixture_conflicting_retry_timeout_policy`)
- 63 unreachable node groups:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`validation_marks_unreachable_node_groups`)
- 64 duplicate node IDs:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`validation_rejects_duplicate_node_ids_and_output_bindings`)
- 65 invalid input binding references:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`validation_rejects_invalid_input_binding_and_missing_environment_reference`)
- 66 unsupported execution mode combinations:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`validation_rejects_unsupported_execution_mode_combinations_and_invalid_tag_filters`)
- 67 illegal output collisions across nodes:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`validation_rejects_duplicate_node_ids_and_output_bindings`)
  - `crates/bijux-dag-core/tests/validation_fixtures.rs` (`fixture_output_collision`)
- 68 invalid selector/tag filter combinations:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`validation_rejects_unsupported_execution_mode_combinations_and_invalid_tag_filters`)
- 69 unsupported runtime capability requirements:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`planner_inclusion_exclusion_and_capability_diagnostics_are_stable`)
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs` (`planner_capability_restrictions_and_dependency_closure_are_enforced`)

## Planner fixtures and contracts (70-80)

- 70 selective replay planning:
  - `crates/bijux-dag-core/tests/snapshots/selective_replay.dag.json`
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
- 71 imported-bundle replay planning:
  - `crates/bijux-dag-core/tests/snapshots/imported_bundle_replay.dag.json`
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
- 72 backend capability rejection:
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs` (`planner_capability_restrictions_and_dependency_closure_are_enforced`)
- 73 resource-heavy DAGs:
  - `crates/bijux-dag-core/tests/snapshots/resource_heavy.dag.json`
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
- 74 retry-heavy DAGs:
  - `crates/bijux-dag-core/tests/snapshots/retry_heavy.dag.json`
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
- 75 replay-heavy DAGs:
  - `crates/bijux-dag-core/tests/snapshots/replay_oriented.dag.json`
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
- 76 plan-explain snapshot tests:
  - `crates/bijux-dag-app/tests/operator_human_snapshot_contracts.rs`
  - `crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs`
- 77 plan error-code contract tests:
  - `crates/bijux-dag-core/tests/planner_error_and_schema_contracts.rs` (`planner_error_codes_contract_is_stable`)
- 78 deterministic plan dump ordering tests:
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs` (`planner_json_dump_and_schema_compatibility_are_stable`)
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`planner_plan_dump_is_deterministic_and_schema_compatible_for_replay_oriented_graph`)
- 79 planner diagnostics for unsupported backend capability paths:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs` (`planner_inclusion_exclusion_and_capability_diagnostics_are_stable`)
- 80 planner hardening suite fast-lane eligible:
  - `configs/suites/planner_identity_closure_fast.json`
  - `crates/bijux-dev-dag/tests/planner_fast_suite_contracts.rs`
