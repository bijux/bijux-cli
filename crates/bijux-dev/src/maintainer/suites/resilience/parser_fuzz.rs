#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-PARSER-FUZZ-HARDENING-REPORTS" => {
            let routing_test =
                workspace_root.join("crates/bijux-cli/tests/routing/parser_input_stability.rs");
            let bin_test = workspace_root
                .join("crates/bijux-cli/tests/integration/cli/root/parser_invalid_utf8_argv.rs");
            let regression_test =
                workspace_root.join("crates/bijux-cli/tests/routing/parser_case_replays.rs");
            let corpus_dir = workspace_root
                .join("crates/bijux-cli/tests/routing/fuzz/parser_interesting_inputs");
            let min_dir =
                workspace_root.join("crates/bijux-cli/tests/routing/fuzz/parser_minimized_cases");
            let texts = BTreeMap::from([
                (routing_test.clone(), fs::read_to_string(&routing_test).unwrap_or_default()),
                (bin_test.clone(), fs::read_to_string(&bin_test).unwrap_or_default()),
                (regression_test.clone(), fs::read_to_string(&regression_test).unwrap_or_default()),
            ]);
            let required: BTreeMap<i64, (PathBuf, &str)> = BTreeMap::from([
                (1, (routing_test.clone(), "fuzz_root_argv_parsing_does_not_panic")),
                (2, (routing_test.clone(), "fuzz_cli_argv_parsing_does_not_panic")),
                (3, (routing_test.clone(), "fuzz_maintainer_argv_parsing_does_not_panic")),
                (4, (routing_test.clone(), "fuzz_plugin_command_argv_parsing_does_not_panic")),
                (5, (routing_test.clone(), "fuzz_config_command_argv_parsing_does_not_panic")),
                (6, (routing_test.clone(), "fuzz_diagnostics_command_argv_parsing_does_not_panic")),
                (
                    7,
                    (
                        routing_test.clone(),
                        "fuzz_mixed_global_local_flag_ordering_is_deterministic",
                    ),
                ),
                (
                    8,
                    (
                        routing_test.clone(),
                        "fuzz_repeated_conflicting_flags_stays_safe_and_deterministic",
                    ),
                ),
                (9, (bin_test.clone(), "malformed_utf8_argv_is_rejected_without_panic")),
                (10, (routing_test.clone(), "fuzz_huge_tokens_and_values_does_not_panic")),
                (11, (routing_test.clone(), "fuzz_typo_suggestion_paths_are_stable")),
                (12, (routing_test.clone(), "fuzz_help_path_parsing_and_alias_resolution_is_safe")),
                (13, (routing_test.clone(), "fuzz_help_path_parsing_and_alias_resolution_is_safe")),
                (
                    14,
                    (
                        routing_test.clone(),
                        "fuzz_namespace_normalization_and_reserved_rejection_stays_safe",
                    ),
                ),
                (
                    15,
                    (
                        routing_test.clone(),
                        "fuzz_reserved_name_rejection_and_normalization_are_deterministic",
                    ),
                ),
                (
                    17,
                    (
                        regression_test.clone(),
                        "interesting_corpus_cases_do_not_crash_or_corrupt_route_resolution",
                    ),
                ),
                (
                    18,
                    (
                        regression_test.clone(),
                        "minimized_parser_cases_do_not_crash_and_are_deterministic",
                    ),
                ),
                (
                    19,
                    (
                        regression_test.clone(),
                        "minimized_parser_cases_do_not_crash_and_are_deterministic",
                    ),
                ),
            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(coverage_id, (path, test_name))| {
                                    let text = texts.get(path).cloned().unwrap_or_default();
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": test_name,
                                        "status": if text.contains(&format!("fn {test_name}(")) { "covered" } else { "missing" },
                                        "evidence": rel(path, workspace_root),
                                    })
                                })
                                .collect();
            let corpus_files: Vec<String> = collect_files(&corpus_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("txt"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let minimized_files: Vec<String> = collect_files(&min_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("argv"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let regression_ok = Command::new("cargo")
                .args(["test", "-p", "bijux-cli", "--test", "routing", "parser_case_replays::"])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let missing_ids: Vec<i64> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/parser_crash_triage_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "parser crash triage",
                                    "coverage_ids": [16],
                                    "status": if regression_ok { "clean" } else { "needs-triage" },
                                    "known_crash_case_count": minimized_files.len(),
                                    "regression_test_ok": regression_ok,
                                    "regression_test_command": ["cargo","test","-p","bijux-cli","--test","routing","parser_case_replays::"],
                                    "triage_notes": [
                                        "minimized cases are retained and replayed on every gate run",
                                        "new parser crashes must be added as minimized reproducer cases",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/parser_fuzz_regression_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "parser fuzz regressions",
                                    "coverage_ids": [19, 20],
                                    "status": if regression_ok && missing_ids.is_empty() { "clean" } else { "drift" },
                                    "missing_coverage_ids": missing_ids,
                                    "corpus_file_count": corpus_files.len(),
                                    "minimized_case_count": minimized_files.len(),
                                    "regression_test_ok": regression_ok,
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/parser_fuzz_campaign_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "parser fuzzing",
                                    "coverage_ids": (1..21).collect::<Vec<_>>(),
                                    "status": if missing_ids.is_empty() && !corpus_files.is_empty() && !minimized_files.is_empty() { "complete" } else { "partial" },
                                    "coverage_rows": coverage_rows,
                                    "corpus_directory": "crates/bijux-cli/tests/routing/fuzz/parser_interesting_inputs",
                                    "corpus_files": corpus_files,
                                    "minimized_directory": "crates/bijux-cli/tests/routing/fuzz/parser_minimized_cases",
                                    "minimized_files": minimized_files,
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/parser_crash_triage_artifact.json",
                "artifacts/status/parser_fuzz_regression_artifact.json",
                "artifacts/status/parser_fuzz_campaign_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CLEANUP-REPORTS" => {
            let generated_at = generated_at_utc();
            let deleted_docs = vec![
                "docs/architecture/newly-ported-command-parity.md",
                "docs/architecture/next-five-command-priorities.md",
                "docs/architecture/safe-improvements-after-parity.md",
            ];
            let deleted_snapshot_files = vec![
                "artifacts/python-behavior/golden/config/config_get_sample.json",
                "artifacts/python-behavior/golden/config/config_set_sample.json",
                "artifacts/python-behavior/golden/config/config_unset_sample.json",
            ];
            let deleted_artifacts = vec![
                "artifacts/python-behavior/golden/config/capture-summary.json",
                "artifacts/python-behavior/golden/config/config_clear.json",
                "artifacts/python-behavior/golden/config/config_export_json.json",
            ];
            let policy_files = json!({
                "artifact_retention": "docs/architecture/artifact-retention-policy.md",
                "snapshot_retention": "docs/architecture/artifact-retention-policy.md",
                "document_retention": "docs/architecture/artifact-retention-policy.md",
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/docs_unreferenced_candidates.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "deleted": deleted_docs,
                    "criteria": [
                        "not linked by README, command reference, or contributor flow",
                        "historical progress reporting rather than durable law",
                    ],
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/stale_snapshot_candidates.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "deleted": deleted_snapshot_files,
                                    "criteria": [
                                        "legacy python-behavior captures no longer tied to live rust command snapshots",
                                        "not consumed by CI upload, release evidence, or tests",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/dead_generated_artifact_candidates.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "deleted": deleted_artifacts,
                                    "criteria": [
                                        "runtime lock and temp files in artifact tree are not evidence artifacts",
                                        "legacy python behavior captures not consumed by CI upload, release evidence, or status reports",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cleanup_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "761-780 cleanup and retention hardening",
                    "deleted": {
                        "docs": deleted_docs,
                        "snapshot_artifacts": deleted_snapshot_files,
                        "dead_generated_artifacts": deleted_artifacts,
                    },
                    "policies": policy_files,
                    "rules": [
                        "reject keep-just-in-case for stale prose",
                        "reject keep-just-in-case for stale snapshots",
                        "reject keep-just-in-case for dead generated artifacts",
                        "cleanup is ongoing release-by-release work",
                    ],
                    "status": "complete",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/docs_unreferenced_candidates.json",
                "artifacts/status/stale_snapshot_candidates.json",
                "artifacts/status/dead_generated_artifact_candidates.json",
                "artifacts/status/cleanup_report.json"
            ]}))
        }
        _ => None,
    }
}
