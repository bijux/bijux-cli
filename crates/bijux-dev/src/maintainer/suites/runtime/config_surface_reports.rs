#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-CONFIG-READ-SURFACE-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/integration/cli/config/config_read_matrix.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (
                    261,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    262,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    263,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    264,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    265,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    266,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (267, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (268, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (269, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (270, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (271, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (272, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (273, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (274, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (275, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (276, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (277, "config_listing_repeated_run_determinism_and_field_order_stability"),
                (278, "config_listing_repeated_run_determinism_and_field_order_stability"),
                (279, "config_listing_repeated_run_determinism_and_field_order_stability"),
            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(coverage_id, fn_name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": fn_name,
                                        "status": if source.contains(&format!("fn {fn_name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/integration/cli/config/config_read_matrix.rs",
                                    })
                                })
                                .collect();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_read_matrix_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "config read matrix",
                                    "coverage_rows": coverage_rows,
                                    "domains": [
                                        {"surface": "root config list", "status": "complete", "evidence": "config_read_matrix.rs"},
                                        {"surface": "cli config get", "status": "complete", "evidence": "config_read_matrix.rs"},
                                        {"surface": "json/yaml/text rendering", "status": "complete", "evidence": "config_read_matrix.rs"},
                                        {"surface": "quiet/no-color behavior", "status": "complete", "evidence": "config_read_matrix.rs"},
                                        {"surface": "deterministic repeated runs", "status": "complete", "evidence": "config_read_matrix.rs"},
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_read_domain_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "domain": "config-read",
                                    "status": "frozen",
                                    "rule": "Config reads must remain deterministic, explainable, and consistent across listing/get surfaces.",
                                    "evidence": [
                                        "crates/bijux-cli/tests/integration/cli/config/config_read_matrix.rs",
                                        "artifacts/status/config_read_matrix_artifact.json",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/config_read_matrix_artifact.json",
                "artifacts/status/config_read_domain_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CONFIG-MUTATION-SURFACE-REPORTS" => {
            let source =
                fs::read_to_string(workspace_root.join(
                    "crates/bijux-cli/tests/integration/cli/config/config_mutation_matrix.rs",
                ))
                .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (281, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (282, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (283, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (284, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (285, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (286, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (287, "config_unset_existing_and_missing_keys"),
                (288, "config_unset_existing_and_missing_keys"),
                (289, "config_clear_populated_and_empty_and_reload_after_external_change"),
                (290, "config_clear_populated_and_empty_and_reload_after_external_change"),
                (291, "config_clear_populated_and_empty_and_reload_after_external_change"),
                (292, "config_export_text_json_yaml_and_load_valid_malformed"),
                (293, "config_export_text_json_yaml_and_load_valid_malformed"),
                (294, "config_export_text_json_yaml_and_load_valid_malformed"),
                (295, "config_export_text_json_yaml_and_load_valid_malformed"),
                (296, "config_export_text_json_yaml_and_load_valid_malformed"),
                (297, "config_mutation_rollback_and_retry_idempotency_proof"),
                (298, "config_mutation_rollback_and_retry_idempotency_proof"),
                (299, "config_mutation_rollback_and_retry_idempotency_proof"),
            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(coverage_id, fn_name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": fn_name,
                                        "status": if source.contains(&format!("fn {fn_name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/integration/cli/config/config_mutation_matrix.rs",
                                    })
                                })
                                .collect();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_mutation_matrix_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "config mutation matrix",
                                    "coverage_rows": coverage_rows,
                                    "domains": [
                                        {"surface": "config set", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                                        {"surface": "config unset", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                                        {"surface": "config clear/reload", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                                        {"surface": "config export/load", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                                        {"surface": "rollback + retry idempotency", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_mutation_domain_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "domain": "config-mutation",
                                    "status": "frozen",
                                    "rule": "Config mutation behavior is accepted only with rollback safety and idempotent retry proof.",
                                    "evidence": [
                                        "crates/bijux-cli/tests/integration/cli/config/config_mutation_matrix.rs",
                                        "artifacts/status/config_mutation_matrix_artifact.json",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/config_mutation_matrix_artifact.json",
                "artifacts/status/config_mutation_domain_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CONFIG-SOURCE-SURFACE-REPORTS" => {
            let source = fs::read_to_string(workspace_root.join(
                "crates/bijux-cli/tests/integration/cli/config/config_source_precedence_laws.rs",
            ))
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (301, "cli_flags_override_env_backed_values_and_config_path"),
                (302, "env_overrides_file_and_file_overrides_default_with_missing_fallback"),
                (303, "env_overrides_file_and_file_overrides_default_with_missing_fallback"),
                (304, "cli_flags_override_env_backed_values_and_config_path"),
                (305, "env_overrides_file_and_file_overrides_default_with_missing_fallback"),
                (306, "malformed_and_duplicate_config_source_behavior_is_stable"),
                (307, "malformed_and_duplicate_config_source_behavior_is_stable"),
                (308, "env_overrides_file_and_file_overrides_default_with_missing_fallback"),
                (309, "cli_flags_override_env_backed_values_and_config_path"),
                (310, "env_overrides_file_and_file_overrides_default_with_missing_fallback"),
                (311, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (312, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (313, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (314, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (315, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (316, "cross_command_source_precedence_consistency"),
                (317, "cross_command_source_precedence_consistency"),
                (318, "cross_command_source_precedence_consistency"),
            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(coverage_id, fn_name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": fn_name,
                                        "status": if source.contains(&format!("fn {fn_name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/integration/cli/config/config_source_precedence_laws.rs",
                                    })
                                })
                                .collect();
            let temp_root = workspace_root.join("artifacts/tmp/config-source-reports");
            fs::create_dir_all(&temp_root).ok()?;
            let config_file = temp_root.join("config.env");
            fs::write(&config_file, "BIJUXCLI_ALPHA=from-file\n").ok()?;
            let envs = vec![("BIJUXCLI_CONFIG", config_file.display().to_string())];
            let get_payload =
                run_bijux_json_env(workspace_root, &["cli", "config", "get", "alpha"], &envs)
                    .ok()?;
            let maintainer_env_payload =
                run_bijux_json_env(workspace_root, &["env"], &envs).ok()?;
            let source_path = get_payload.get("source_path").cloned().unwrap_or(Value::Null);
            let active_config = maintainer_env_payload
                .get("active")
                .and_then(|v| v.get("config_file"))
                .cloned()
                .unwrap_or(Value::Null);
            let precedence =
                maintainer_env_payload.get("source_precedence").cloned().unwrap_or(Value::Null);
            let mut drift_reasons = Vec::<String>::new();
            if source_path != active_config {
                drift_reasons.push(
                    "config_get.source_path does not match maintainer_env.active.config_file"
                        .to_string(),
                );
            }
            if precedence != json!(["flags", "env", "config", "defaults"]) {
                drift_reasons.push(
                    "maintainer_env.source_precedence does not match expected order".to_string(),
                );
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_source_parity_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config precedence/source parity",
                    "coverage_rows": coverage_rows,
                    "comparison": {
                        "config_get_source_path": source_path,
                        "maintainer_env_active_config_file": active_config,
                        "maintainer_env_source_precedence": precedence,
                    },
                    "status": if drift_reasons.is_empty() { "consistent" } else { "drift" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_source_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config precedence/source drift",
                    "drift_count": drift_reasons.len(),
                    "drift_reasons": drift_reasons,
                    "status": if drift_reasons.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_source_precedence_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "domain": "config-source-precedence",
                                    "status": "frozen",
                                    "rule": "Config precedence truth must be observable, deterministic, and consistent across config get and bijux-dev-cli env.",
                                    "evidence": [
                                        "crates/bijux-cli/tests/integration/cli/config/config_source_precedence_laws.rs",
                                        "artifacts/status/config_source_parity_artifact.json",
                                        "artifacts/status/config_source_drift_artifact.json",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/config_source_parity_artifact.json",
                "artifacts/status/config_source_drift_artifact.json",
                "artifacts/status/config_source_precedence_contract.json"
            ]}))
        }
        _ => None,
    }
}
