# Evidence Shared Asset Usage

This report tracks where canonical evidence assets are consumed across suite surfaces.

## Shared Usage Rules
- Keep one canonical asset when semantics are identical across consumers.
- Duplicate assets when consumer semantics diverge or diagnostics must differ by domain.
- Shared usage must stay explicit in metadata and pass drift checks.

## Multi-Consumer Assets

| Asset | Kind | Consumers |
| --- | --- | --- |
| `evidence/authoring/examples/cached-branched-report.dag.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/examples/etl-constant-to-shell.dag.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/examples/failure-heavy-retry.dag.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/examples/hello.dag.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/examples/multi-output-artifact.dag.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/examples/replay-heavy-branching.dag.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/negative/cycle.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/negative/invalid_container_workdir.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/negative/invalid_refs.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/negative/invalid_selectors.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/negative/missing_required_input_binding.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/negative/undeclared_outputs.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/negative/unsupported_adapter_payload.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/patterns/medium.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/patterns/minimal.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/patterns/pattern_aggregation.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/patterns/pattern_cache_heavy.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/patterns/pattern_chain.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/patterns/pattern_diamond.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/patterns/pattern_fanout.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/authoring/patterns/pattern_replay_sensitive.json` | `authoring` | `dag-validate, authoring-contracts` |
| `evidence/battle/workflows/cache/cache_hit_second_run.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/cache/fingerprint_change_invalidates_cache.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/container_execution_if_supported.json` | `battle` | `e2e-matrix, battle-suite` |
| `evidence/battle/workflows/e2e_minimal.json` | `battle` | `e2e-contracts, runtime-tests` |
| `evidence/battle/workflows/failure/missing_outputs_rejected.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/failure/node_failure_downstream_skipped.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/failure/retry_then_success_accounting.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/failure/timeout_error_classification.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/failure/validation_failure_has_no_partial_run_dir.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/happy_path/diamond_outputs_and_manifest_totals.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/happy_path/parse_validate_run_inspect_replay.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/happy_path/real_world_orchestration_stress.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/import_export/export_import_metadata_only.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/import_export/export_import_with_files.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/policy/deny_env_clean_env_allowlist.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/replay/replay_semantic_comparison.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/artifact_heavy_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/branch_join_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/cache_invalidation_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/corruption_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/failure_heavy_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/import_export_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/large_dag_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/malformed_run_dir_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/medium_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/multi_root_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/operator_debugging_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/operator_inspection_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/policy_violation_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/replay_divergence_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/resource_contention_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/retry_storm_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/scheduler_fairness_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/secret_leakage_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/timeout_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/ugly_realistic_dag_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/runtime/version_compatibility_workflow.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/battle/workflows/selection/include_exclude_filters.json` | `battle` | `crate-contracts, runtime-tests` |
| `evidence/cache/corrupt/hash_mismatch.json` | `cache` | `cache-contracts, runtime-tests` |
| `evidence/cache/corrupt/missing_manifest.json` | `cache` | `cache-contracts, runtime-tests` |
| `evidence/cache/corrupt/missing_meta.json` | `cache` | `cache-contracts, runtime-tests` |
| `evidence/cache/corrupt/missing_outputs_proof.json` | `cache` | `cache-contracts, runtime-tests` |
| `evidence/cache/corrupt/truncated_meta.json` | `cache` | `cache-contracts, runtime-tests` |
| `evidence/cache/corrupt/unsupported_metadata_version.json` | `cache` | `cache-contracts, runtime-tests` |
| `evidence/cache/replay/corruption_case.json` | `cache` | `e2e-contracts, runtime-tests` |
| `evidence/cache/replay/match_case.json` | `cache` | `e2e-contracts, runtime-tests` |
| `evidence/cache/replay/mismatch_case.json` | `cache` | `e2e-contracts, runtime-tests` |
| `evidence/cache/replay/unsupported_version_case.json` | `cache` | `e2e-contracts, runtime-tests` |
| `evidence/cache/scenarios/warm_cold.json` | `cache` | `cache-contracts, runtime-tests` |
| `evidence/compat/export_bundle/unsupported_past/bundle.json` | `compat` | `crate-contracts, runtime-tests` |
| `evidence/compat/export_bundle/v0_1_supported/bundle.json` | `compat` | `crate-contracts, runtime-tests` |
| `evidence/compat/graph_schema/unsupported_future/minimal.dag.json` | `compat` | `crate-contracts, runtime-tests` |
| `evidence/compat/graph_schema/unsupported_past/minimal.dag.json` | `compat` | `crate-contracts, runtime-tests` |
| `evidence/compat/graph_schema/v0_1_supported/minimal.dag.json` | `compat` | `crate-contracts, runtime-tests` |
| `evidence/compat/run_dir/unsupported_future/manifest.json` | `compat` | `crate-contracts, runtime-tests` |
| `evidence/compat/run_dir/v0_1_supported/manifest.json` | `compat` | `crate-contracts, runtime-tests` |
| `evidence/compat/scenarios/historical_fixture_validation.json` | `compat` | `compatibility-contracts, e2e-matrix` |
| `evidence/fault/classes/fault_classes.json` | `fault` | `fault-contracts, runtime-tests` |
| `evidence/fault/corrupt_runs/invalid_outputs_index.json` | `fault` | `crate-contracts, runtime-tests` |
| `evidence/fault/corrupt_runs/missing_manifest_version.json` | `fault` | `crate-contracts, runtime-tests` |
| `evidence/operator/scenarios/inspection_only.json` | `operator` | `crate-contracts, runtime-tests` |
| `evidence/perf/scenarios/cache_heavy_canonical.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/cache_heavy_lookup.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/cache_metadata_growth.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/deep_canonical.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/deep_scheduler_overhead.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/few_heavy_nodes_orchestration_overhead.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/manifest_trace_volume_growth.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/manifest_trace_write_amplification.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/many_small_nodes_scheduler_overhead.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/medium_canonical.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/medium_execute_local.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/memory_execute_many_nodes.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/memory_import_export_path.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/memory_parse_validate_large_graph.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/memory_replay_path.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/replay_canonical.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/replay_verification_cost.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/resource_budgets.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/tiny_canonical.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/tiny_parse_validate.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/wide_canonical.json` | `perf` | `benchmark-suite, performance-evidence` |
| `evidence/perf/scenarios/wide_scheduler_overhead.json` | `perf` | `benchmark-suite, performance-evidence` |
