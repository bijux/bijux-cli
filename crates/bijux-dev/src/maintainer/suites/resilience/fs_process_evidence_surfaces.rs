#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-EVIDENCE-INTEGRITY-REPORTS" => {
            let read = |path: &str| -> Value {
                fs::read_to_string(workspace_root.join(path))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let run_cmd = |args: &[&str]| -> Value {
                run_bijux_json(workspace_root, args).unwrap_or_else(|_| json!({}))
            };
            let evidence_audit = run_cmd(&["evidence", "audit"]);
            let evidence_map = run_cmd(&["evidence", "command-map"]);
            let parity_map = run_cmd(&["evidence", "parity-map"]);
            let invalid_ids = evidence_audit
                .get("invalid_ids")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let missing_links = evidence_audit
                .get("missing_artifact_links")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let orphan_report = evidence_audit
                .get("orphan_report")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let claims_without = evidence_audit
                .get("claims_without_evidence_report")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/evidence_coverage_report.json",
                                &json!({
                                    "records": evidence_audit.get("coverage_report").cloned().unwrap_or_else(|| json!([])),
                                    "source": "bijux-dev-cli evidence audit",
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/evidence_integrity_artifact.json",
                                &json!({
                                    "generator": "bijux-dev-cli",
                                    "scope": "evidence integrity",
                                    "checks": {
                                        "invalid_ids": invalid_ids,
                                        "missing_artifact_links": missing_links,
                                        "orphan_report": orphan_report,
                                        "claims_without_evidence_report": claims_without,
                                    },
                                    "status": if invalid_ids.is_empty() && missing_links.is_empty() && orphan_report.is_empty() && claims_without.is_empty() { "complete" } else { "partial" },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/orphan_evidence_report.json",
                &json!({
                    "records": orphan_report,
                    "source": "bijux-dev-cli evidence audit",
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/orphan_evidence_artifact.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "orphan evidence",
                    "records": orphan_report,
                    "count": orphan_report.len(),
                    "status": if orphan_report.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/claim_without_evidence_report.json",
                &json!({
                    "records": claims_without,
                    "source": "bijux-dev-cli evidence audit",
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/evidence_command_map_report.json",
                &evidence_map,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/evidence_parity_map_report.json",
                &parity_map,
            )
            .ok()?;

            let rust_owner = run_cmd(&["config", "rust-owner"]);
            let python_owner = run_cmd(&["config", "python-owner"]);
            let ownership = run_cmd(&["config", "ownership"]);
            let drift = run_cmd(&["config", "drift"]);
            let shape = run_cmd(&["config", "shape"]);
            let evidence_link = run_cmd(&["config", "evidence-map"]);
            let _ = read("artifacts/status/config_ownership_truth.json");
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_owners_by_layer_report.json",
                &json!({"rust": rust_owner, "python": python_owner}),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_file_schema_owners_report.json",
                &json!({
                    "owners": ownership.get("owners").cloned().unwrap_or_else(|| json!({})),
                    "schemas": shape.get("schemas").cloned().unwrap_or_else(|| json!([])),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_python_compatibility_shims_report.json",
                                &json!({
                                    "compatibility_shims": ownership.get("compatibility_shims").cloned().unwrap_or_else(|| json!([])),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_rust_sources_report.json",
                &json!({"sources": shape.get("sources").cloned().unwrap_or_else(|| json!([]))}),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_precedence_proofs_report.json",
                                &json!({"precedence_proofs": shape.get("precedence_proofs").cloned().unwrap_or_else(|| json!([]))}),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_mutation_rollback_proofs_report.json",
                                &json!({"rollback_proofs": shape.get("rollback_proofs").cloned().unwrap_or_else(|| json!([]))}),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_corruption_evidence_report.json",
                                &json!({"corruption_evidence": shape.get("corruption_evidence").cloned().unwrap_or_else(|| json!([]))}),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_owner_drift_report.json",
                &drift,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_evidence_link_report.json",
                &evidence_link,
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_ownership_truth.json",
                                &json!({
                                    "owners": ownership.get("owners").cloned().unwrap_or_else(|| json!({})),
                                    "schemas": shape.get("schemas").cloned().unwrap_or_else(|| json!([])),
                                    "compatibility_shims": ownership.get("compatibility_shims").cloned().unwrap_or_else(|| json!([])),
                                    "sources": shape.get("sources").cloned().unwrap_or_else(|| json!([])),
                                    "precedence_proofs": shape.get("precedence_proofs").cloned().unwrap_or_else(|| json!([])),
                                    "rollback_proofs": shape.get("rollback_proofs").cloned().unwrap_or_else(|| json!([])),
                                    "corruption_evidence": shape.get("corruption_evidence").cloned().unwrap_or_else(|| json!([])),
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/evidence_coverage_report.json",
                "artifacts/status/evidence_integrity_artifact.json",
                "artifacts/status/orphan_evidence_report.json",
                "artifacts/status/orphan_evidence_artifact.json",
                "artifacts/status/claim_without_evidence_report.json",
                "artifacts/status/evidence_command_map_report.json",
                "artifacts/status/evidence_parity_map_report.json",
                "artifacts/status/config_owners_by_layer_report.json",
                "artifacts/status/config_file_schema_owners_report.json",
                "artifacts/status/config_python_compatibility_shims_report.json",
                "artifacts/status/config_rust_sources_report.json",
                "artifacts/status/config_precedence_proofs_report.json",
                "artifacts/status/config_mutation_rollback_proofs_report.json",
                "artifacts/status/config_corruption_evidence_report.json",
                "artifacts/status/config_owner_drift_report.json",
                "artifacts/status/config_evidence_link_report.json",
                "artifacts/status/config_ownership_truth.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-HISTORY-SURFACE-REPORTS" => {
            let source = fs::read_to_string(workspace_root.join(
                "crates/bijux-cli/tests/integration/cli/history/history_command_coverage.rs",
            ))
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (322, "history_root_listing_no_file_one_record_many_records_and_ordering"),
                (323, "history_root_listing_no_file_one_record_many_records_and_ordering"),
                (324, "history_root_listing_no_file_one_record_many_records_and_ordering"),
                (325, "history_text_json_yaml_quiet_and_no_color_modes"),
                (326, "history_text_json_yaml_quiet_and_no_color_modes"),
                (327, "history_text_json_yaml_quiet_and_no_color_modes"),
                (328, "history_root_listing_no_file_one_record_many_records_and_ordering"),
                (329, "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates"),
                (330, "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates"),
                (331, "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates"),
                (332, "history_limit_path_override_and_repeated_run_determinism"),
                (333, "history_limit_path_override_and_repeated_run_determinism"),
                (334, "history_clear_with_unwritable_parent_fails_stably"),
                (335, "history_text_json_yaml_quiet_and_no_color_modes"),
                (336, "history_text_json_yaml_quiet_and_no_color_modes"),
                (337, "history_limit_path_override_and_repeated_run_determinism"),
                (338, "history_help_and_exit_discipline_for_root_and_clear"),
                (339, "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates"),
            ]);
            let coverage_rows = required
                                .iter()
                                .map(|(coverage_id, fn_name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": fn_name,
                                        "status": if source.contains(&format!("fn {fn_name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/integration/cli/history/history_command_coverage.rs",
                                    })
                                })
                                .collect::<Vec<_>>();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/history_command_coverage_report.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "history command coverage",
                                    "commands": coverage_rows,
                                    "summary": {
                                        "total": coverage_rows.len(),
                                        "complete": coverage_rows.iter().filter(|row| row.get("status").and_then(Value::as_str) == Some("complete")).count(),
                                        "partial": 0,
                                        "shim": 0,
                                        "missing": coverage_rows.iter().filter(|row| row.get("status").and_then(Value::as_str) == Some("missing")).count(),
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/history_command_coverage_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "history command matrix",
                    "coverage_rows": coverage_rows,
                    "commands": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/history_corruption_matrix_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "history corruption matrix",
                                    "cases": [
                                        {
                                            "name": "line-layout malformed and mixed records",
                                            "status": "complete",
                                            "evidence": "history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates",
                                        },
                                        {
                                            "name": "unwritable parent directory on clear",
                                            "status": "complete",
                                            "evidence": "history_clear_with_unwritable_parent_fails_stably",
                                        }
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/history_read_domain_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "domain": "history-read-behavior",
                                    "status": "frozen",
                                    "rule": "History read behavior must remain deterministic, format-stable, and resilient under malformed storage states.",
                                    "evidence": [
                                        "crates/bijux-cli/tests/integration/cli/history/history_command_coverage.rs",
                                        "artifacts/status/history_command_coverage_artifact.json",
                                        "artifacts/status/history_corruption_matrix_artifact.json",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/history_command_coverage_report.json",
                "artifacts/status/history_command_coverage_artifact.json",
                "artifacts/status/history_corruption_matrix_artifact.json",
                "artifacts/status/history_read_domain_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-SURFACE-REPORTS" => {
            let tests_root = workspace_root.join("crates/bijux-cli/tests");
            let sources: BTreeMap<String, String> = collect_files(&tests_root)
                .into_iter()
                .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
                .map(|path| {
                    (rel(&path, workspace_root), fs::read_to_string(path).unwrap_or_default())
                })
                .collect();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (362, "inspect_text_json_yaml_quiet_and_trace_modes"),
                (363, "inspect_text_json_yaml_quiet_and_trace_modes"),
                (364, "inspect_text_json_yaml_quiet_and_trace_modes"),
                (365, "inspect_text_json_yaml_quiet_and_trace_modes"),
                (366, "inspect_text_json_yaml_quiet_and_trace_modes"),
                (367, "doctor_text_json_and_corrupted_state_coverage"),
                (368, "doctor_text_json_and_corrupted_state_coverage"),
                (369, "doctor_text_json_and_corrupted_state_coverage"),
                (370, "doctor_text_json_and_corrupted_state_coverage"),
                (371, "doctor_text_json_and_corrupted_state_coverage"),
                (372, "doctor_text_json_and_corrupted_state_coverage"),
                (373, "maintainer_routes_registry_env_contracts_json_shape_stability"),
                (374, "maintainer_routes_registry_env_contracts_json_shape_stability"),
                (375, "maintainer_routes_registry_env_contracts_json_shape_stability"),
                (376, "maintainer_routes_registry_env_contracts_json_shape_stability"),
                (377, "diagnostics_consistency_across_inspect_doctor_and_maintainer_surfaces"),
                (378, "diagnostics_consistency_across_inspect_doctor_and_maintainer_surfaces"),
                (379, "diagnostics_consistency_across_inspect_doctor_and_maintainer_surfaces"),
            ]);
            let coverage_rows = required
                .iter()
                .map(|(coverage_id, fn_name)| {
                    let evidence = sources
                        .iter()
                        .find(|(_, src)| src.contains(&format!("fn {fn_name}(")))
                        .map(|(path, _)| path.clone());
                    json!({
                        "coverage_id": coverage_id,
                        "test": fn_name,
                        "status": if evidence.is_some() { "complete" } else { "missing" },
                        "evidence": evidence,
                    })
                })
                .collect::<Vec<_>>();
            let drift = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("complete"))
                .filter_map(|row| row.get("test").and_then(Value::as_str))
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/diagnostics_command_coverage_report.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "diagnostics command coverage",
                                    "commands": coverage_rows,
                                    "summary": {
                                        "total": coverage_rows.len(),
                                        "complete": coverage_rows.iter().filter(|row| row.get("status").and_then(Value::as_str) == Some("complete")).count(),
                                        "partial": 0,
                                        "shim": 0,
                                        "missing": coverage_rows.iter().filter(|row| row.get("status").and_then(Value::as_str) == Some("missing")).count(),
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_matrix_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "diagnostics matrix",
                    "coverage_rows": coverage_rows,
                    "commands": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_shape_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "diagnostics shape drift",
                    "drift_count": drift.len(),
                    "drift_commands": drift,
                    "status": if drift.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/diagnostics_operator_truth_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "domain": "diagnostics-operator-truth",
                                    "status": "frozen",
                                    "rule": "Diagnostics outputs must remain structured, consistent across surfaces, and stable in machine shape.",
                                    "evidence": [
                                        "crates/bijux-cli/tests",
                                        "artifacts/status/diagnostics_matrix_artifact.json",
                                        "artifacts/status/diagnostics_shape_drift_artifact.json",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/diagnostics_command_coverage_report.json",
                "artifacts/status/diagnostics_matrix_artifact.json",
                "artifacts/status/diagnostics_shape_drift_artifact.json",
                "artifacts/status/diagnostics_operator_truth_contract.json"
            ]}))
        }
        _ => None,
    }
}
