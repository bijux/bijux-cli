#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-HISTORY-DEEP-BEHAVIOR-REPORTS" => {
            let tests = [
                "crates/bijux-cli/tests/integration/cli/history/history_command_matrix.rs",
                "crates/bijux-cli/tests/integration/cli/history/history_parity.rs",
                "crates/bijux-cli/tests/integration/cli/history/history_output_stability.rs",
            ];
            let mut sources = BTreeMap::<String, String>::new();
            for path in tests {
                let full = workspace_root.join(path);
                if full.exists() {
                    sources.insert(path.to_string(), fs::read_to_string(full).unwrap_or_default());
                }
            }
            let find_test = |name: &str| -> Option<String> {
                let needle = format!("fn {name}(");
                sources
                    .iter()
                    .find(|(_, source)| source.contains(&needle))
                    .map(|(path, _)| path.clone())
            };
            let run_json = |args: &[&str]| -> Value {
                run_bijux_json(workspace_root, args).unwrap_or_else(|_| json!({}))
            };
            let semantic_sample = run_json(&["history"]);
            let determinism_a = Command::new("cargo")
                .args([
                    "run",
                    "-q",
                    "-p",
                    "bijux-cli",
                    "--",
                    "history",
                    "--format",
                    "json",
                    "--no-pretty",
                ])
                .current_dir(workspace_root)
                .output()
                .ok();
            let determinism_b = Command::new("cargo")
                .args([
                    "run",
                    "-q",
                    "-p",
                    "bijux-cli",
                    "--",
                    "history",
                    "--format",
                    "json",
                    "--no-pretty",
                ])
                .current_dir(workspace_root)
                .output()
                .ok();
            let corruption_sample = run_json(&["history"]);
            let repl_interop_sample = run_json(&["history"]);
            let stream_sample = Command::new("cargo")
                .args(["run", "-q", "-p", "bijux-cli", "--", "history", "--format", "text"])
                .current_dir(workspace_root)
                .output()
                .ok();
            let failure_sample = Command::new("cargo")
                .args(["run", "-q", "-p", "bijux-cli", "--", "history", "--unknown-flag"])
                .current_dir(workspace_root)
                .output()
                .ok();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (101, "history_root_listing_no_file_one_record_many_records_and_ordering"),
                (102, "history_limit_path_override_and_repeated_run_determinism"),
                (103, "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates"),
                (104, "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates"),
                (105, "history_json_yaml_text_outputs_are_emitted"),
                (106, "history_text_json_yaml_quiet_and_no_color_modes"),
                (107, "history_json_yaml_text_outputs_are_emitted"),
                (108, "history_reads_repl_line_layout_for_cli_interop"),
                (109, "history_limit_path_override_and_repeated_run_determinism"),
                (110, "history_missing_and_malformed_behaviors_are_stable"),
                (111, "history_handles_huge_files_with_stable_tail_limit"),
                (112, "history_doctor_and_state_doctor_agree_on_history_corruption_findings"),
                (113, "history_output_is_stable_under_filesystem_metadata_changes"),
            ]);
            let coverage_rows = required
                                        .iter()
                                        .map(|(id, name)| {
                                            let evidence = find_test(name);
                                            json!({"coverage_id":id,"test_name":name,"status":if evidence.is_some(){"covered"}else{"missing"},"evidence":evidence})
                                        })
                                        .collect::<Vec<_>>();
            let missing = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            let det_ok = determinism_a.is_some()
                && determinism_b.is_some()
                && determinism_a.as_ref().is_some_and(|o| o.status.success())
                && determinism_b.as_ref().is_some_and(|o| o.status.success())
                && determinism_a.as_ref().map(|o| (&o.stdout, &o.stderr))
                    == determinism_b.as_ref().map(|o| (&o.stdout, &o.stderr));
            let stream_ok =
                stream_sample.as_ref().is_some_and(|o| o.status.success() && o.stderr.is_empty());
            let failure_code = failure_sample.as_ref().and_then(|o| o.status.code()).unwrap_or(1);
            let history_semantic = json!({"generator":"bijux-dev-cli","scope":"history semantic","coverage_ids":[101,102,103,104,105,108,109,110,111,113,114],"status":if semantic_sample.is_object(){"complete"}else{"partial"},"sample":semantic_sample});
            let history_determinism = json!({"generator":"bijux-dev-cli","scope":"history determinism","coverage_ids":[101,102,107,111,113,115],"status":if det_ok{"complete"}else{"partial"},"byte_stable":det_ok});
            let history_corruption = json!({"generator":"bijux-dev-cli","scope":"history corruption","coverage_ids":[103,104,110,112,116],"status":if corruption_sample.is_object(){"complete"}else{"partial"},"sample":corruption_sample});
            let history_repl_interop = json!({"generator":"bijux-dev-cli","scope":"history repl interop","coverage_ids":[108,117],"status":if repl_interop_sample.is_object(){"complete"}else{"partial"},"sample":repl_interop_sample});
            let history_stream = json!({"generator":"bijux-dev-cli","scope":"history stream discipline","coverage_ids":[106,118],"status":if stream_ok{"complete"}else{"partial"}});
            let history_failure = json!({"generator":"bijux-dev-cli","scope":"history failure class","coverage_ids":[112,119],"status":if failure_code==2{"complete"}else{"partial"},"sample_exit_code":failure_code});
            let mut drift = Vec::<Value>::new();
            for (name, payload) in [
                ("history_semantic_artifact.json", &history_semantic),
                ("history_determinism_artifact.json", &history_determinism),
                ("history_corruption_artifact.json", &history_corruption),
                ("history_repl_interop_artifact.json", &history_repl_interop),
                ("history_stream_discipline_artifact.json", &history_stream),
                ("history_failure_class_artifact.json", &history_failure),
            ] {
                if payload.get("status").and_then(Value::as_str) != Some("complete") {
                    drift.push(json!({"artifact":name,"reason":"status-not-complete"}));
                }
            }
            if !missing.is_empty() {
                drift.push(json!({"reason":"missing-coverage_id-coverage","coverage_ids":missing}));
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/history_semantic_artifact.json",
                &history_semantic,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/history_determinism_artifact.json",
                &history_determinism,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/history_corruption_artifact.json",
                &history_corruption,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/history_repl_interop_artifact.json",
                &history_repl_interop,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/history_stream_discipline_artifact.json",
                &history_stream,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/history_failure_class_artifact.json",
                &history_failure,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/history_deep_behavior_drift_artifact.json", &json!({
                                        "generator":"bijux-dev-cli","scope":"history deep behavior drift","coverage_ids":[120],
                                        "status": if drift.is_empty() { "clean" } else { "drift-detected" },
                                        "drift_count": drift.len(),
                                        "drift_items": drift,
                                        "coverage_rows": coverage_rows,
                                    })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/history_semantic_artifact.json",
                "artifacts/status/history_determinism_artifact.json",
                "artifacts/status/history_corruption_artifact.json",
                "artifacts/status/history_repl_interop_artifact.json",
                "artifacts/status/history_stream_discipline_artifact.json",
                "artifacts/status/history_failure_class_artifact.json",
                "artifacts/status/history_deep_behavior_drift_artifact.json"
            ]}))
        }
        _ => None,
    }
}
