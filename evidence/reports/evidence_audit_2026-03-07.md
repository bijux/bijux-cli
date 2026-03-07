# Evidence Audit 2026-03-07

## Scope

This audit closes the current hardening cycle after evidence-root migration, release-set normalization, and consumer cleanup.

## All evidence assets by family

Source: `evidence/_meta/registries/evidence_registry.json`

| Family | Asset count |
| --- | ---: |
| authoring | 20 |
| battle | 53 |
| cache | 10 |
| compare | 11 |
| compat | 8 |
| fault | 3 |
| operator | 1 |
| perf | 22 |
| total | 128 |

## Release-blocking assets by family

Source: registry `release_blocking=true`.

| Family | Asset count |
| --- | ---: |
| battle | 51 |
| cache | 10 |
| compat | 8 |
| fault | 3 |
| operator | 1 |
| perf | 1 |
| total | 74 |

## Advisory assets by family

Source: `evidence/release/release_evidence_set.json` advisory set.

| Family | Asset count |
| --- | ---: |
| compare | 2 |
| total | 2 |

## Strongest evidence assets (top 20)

Selection rule: release-blocking assets that directly protect deterministic execution, replay/cache truth, operator inspectability, compatibility acceptance, and corruption detection.

1. `evidence/battle/workflows/happy_path/parse_validate_run_inspect_replay.json`
2. `evidence/battle/workflows/happy_path/diamond_outputs_and_manifest_totals.json`
3. `evidence/battle/workflows/adversarial/concurrency_retry_determinism_stress.json`
4. `evidence/battle/workflows/adversarial/cache_proof_corruption_plausible_outputs.json`
5. `evidence/battle/workflows/adversarial/replay_semantic_divergence_detection.json`
6. `evidence/battle/workflows/adversarial/operator_inspection_only_recovery.json`
7. `evidence/battle/workflows/adversarial/policy_denial_of_unsafe_execution.json`
8. `evidence/battle/workflows/adversarial/path_escape_attempt_declared_outputs.json`
9. `evidence/battle/workflows/adversarial/env_leakage_attempt_via_adapter.json`
10. `evidence/battle/workflows/adversarial/partial_run_dir_not_finalized.json`
11. `evidence/battle/workflows/adversarial/imported_run_visibility_contract.json`
12. `evidence/cache/replay/match_case.json`
13. `evidence/cache/replay/mismatch_case.json`
14. `evidence/cache/replay/corruption_case.json`
15. `evidence/compat/graph_schema/v0_1_supported/minimal.dag.json`
16. `evidence/fault/classes/fault_classes.json`
17. `evidence/fault/corrupt_runs/invalid_outputs_index.json`
18. `evidence/fault/corrupt_runs/missing_manifest_version.json`
19. `evidence/operator/scenarios/inspection_only.json`
20. `evidence/perf/scenarios/tiny_canonical.json`

## Weakest evidence assets still present (top 20)

Selection rule: advisory-only assets or transitional perf surfaces with low release value relative to stronger overlapping battle/replay/fault coverage.

1. `evidence/compare/scenarios/chain.json`
2. `evidence/compare/scenarios/diamond.json`
3. `evidence/compare/scenarios/cache_reuse_shape.json`
4. `evidence/compare/scenarios/scheduler_tiny_tasks_overhead.json`
5. `evidence/compare/scenarios/retry_timeout.json`
6. `evidence/compare/scenarios/failure_diagnostics.json`
7. `evidence/perf/scenarios/cache_heavy_lookup.json`
8. `evidence/perf/scenarios/cache_metadata_growth.json`
9. `evidence/perf/scenarios/deep_scheduler_overhead.json`
10. `evidence/perf/scenarios/few_heavy_nodes_orchestration_overhead.json`
11. `evidence/perf/scenarios/manifest_trace_volume_growth.json`
12. `evidence/perf/scenarios/medium_execute_local.json`
13. `evidence/perf/scenarios/memory_import_export_path.json`
14. `evidence/perf/scenarios/memory_parse_validate_large_graph.json`
15. `evidence/perf/scenarios/memory_replay_path.json`
16. `evidence/perf/scenarios/resource_budgets.json`
17. `evidence/perf/scenarios/tiny_parse_validate.json`
18. `evidence/perf/scenarios/wide_canonical.json`
19. `evidence/perf/scenarios/wide_scheduler_overhead.json`
20. `evidence/perf/scenarios/manifest_trace_write_amplification.json`

## Deletions in this audit wave

- Deleted stale audit artifact: `evidence/audit/shallow_evidence_audit_2026-03-07.md`
- Deleted stale scope scan artifact: `evidence/reports/speculative_assets_report.md`

## Review decision

- Evidence architecture remains frozen for one review cycle.
- New evidence growth remains blocked until review-cycle closure criteria are met.
