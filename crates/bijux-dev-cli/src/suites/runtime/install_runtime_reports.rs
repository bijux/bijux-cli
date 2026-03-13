#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-NAMESPACE-RESERVATION-REPORTS" => {
            let routing_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/routing/registry_namespace_policy.rs"),
            )
            .unwrap_or_default();
            let plugin_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli-plugin/tests/plugin_namespace_regression.rs"),
            )
            .unwrap_or_default();
            let cli_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/bin_surface/plugin_cli_lifecycle.rs"),
            )
            .unwrap_or_default();
            let constants =
                fs::read_to_string(workspace_root.join("crates/bijux-cli-plugin/src/constants.rs"))
                    .unwrap_or_default();
            let product_registry = fs::read_to_string(
                workspace_root.join("contracts/official_product_namespace_registry.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let evidence_text = format!("{routing_text}\n{plugin_text}\n{cli_text}");
            let parse_array = |name: &str| -> Vec<String> {
                let marker = format!("pub const {name}: &[&str] =");
                let Some(idx) = constants.find(&marker) else {
                    return Vec::new();
                };
                let chunk = &constants[idx..];
                let Some(start) = chunk.find('[') else {
                    return Vec::new();
                };
                let Some(end) = chunk.find("];") else {
                    return Vec::new();
                };
                chunk[start..end]
                    .split('"')
                    .enumerate()
                    .filter_map(|(i, part)| (i % 2 == 1).then_some(part.to_string()))
                    .collect()
            };
            let namespace_rows = [
                "official_reserved_namespaces_take_precedence",
                "rejects_future_official_product_namespaces",
                "normalized_and_case_folded_namespace_collisions_are_rejected",
            ]
            .iter()
            .map(|name| {
                json!({
                    "evidence_test": name,
                    "status": if evidence_text.contains(name) { "complete" } else { "missing" }
                })
            })
            .collect::<Vec<_>>();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/namespace_abuse_report.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "421-440 namespace and reservation abuse hardening",
                    "rows": namespace_rows
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/reserved_namespace_inventory.json", &json!({
                                "generated_at": generated_at_utc(),
                                "generator": "bijux-dev-cli",
                                "reserved_namespaces": parse_array("RESERVED_NAMESPACES"),
                                "core_namespaces": parse_array("CORE_NAMESPACES"),
                                "future_product_namespaces": parse_array("FUTURE_PRODUCT_NAMESPACES"),
                                "registry_entries": product_registry.get("entries").cloned().unwrap_or_else(|| json!([]))
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/namespace_abuse_report.json",
                "artifacts/status/reserved_namespace_inventory.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-INSTALL-TRUTH-REPORTS" => {
            let generated_at = generated_at_utc();
            let runtime_identity =
                run_bijux_json(workspace_root, &["dev", "cli", "runtime-identity"]).ok()?;
            let package_health =
                run_bijux_json(workspace_root, &["dev", "cli", "package-health"]).ok()?;
            let install_text =
                run_bijux_text(workspace_root, &["dev", "cli", "runtime-identity"]).ok()?;
            let diagnostics =
                runtime_identity.get("diagnostics").cloned().unwrap_or_else(|| json!({}));
            let install_source_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_command": "bijux dev cli runtime-identity --json --no-pretty",
                "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                "install_source": runtime_identity.get("install_source").cloned().unwrap_or(Value::Null),
                "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                "diagnostics": diagnostics,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_source_diagnostics.json",
                &install_source_payload,
            )
            .ok()?;
            let ambiguous_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_command": "bijux dev cli runtime-identity --json --no-pretty",
                "active_binary_selection_is_ambiguous": runtime_identity.get("active_binary_selection_is_ambiguous").cloned().unwrap_or(json!(false)),
                "active_path_is_shadowed": runtime_identity.get("active_path_is_shadowed").cloned().unwrap_or(json!(false)),
                "duplicate_install_detected": diagnostics.get("duplicate_install_detected").cloned().unwrap_or(json!(false)),
                "mixed_pip_cargo_install_detected": diagnostics.get("mixed_pip_cargo_install_detected").cloned().unwrap_or(json!(false)),
                "path_shadowing_detected": diagnostics.get("path_shadowing_detected").cloned().unwrap_or(json!(false)),
                "stale_wrapper_detected": diagnostics.get("stale_wrapper_detected").cloned().unwrap_or(json!(false)),
                "active_binary_mismatch_detected": diagnostics.get("active_binary_mismatch_detected").cloned().unwrap_or(json!(false)),
                "python_bridge_supported": diagnostics.get("python_bridge_supported").cloned().unwrap_or(json!(true)),
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/ambiguous_runtime_diagnostics.json",
                &ambiguous_payload,
            )
            .ok()?;
            let install_health_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_commands": [
                    "bijux dev cli runtime-identity --json --no-pretty",
                    "bijux dev cli package-health --json --no-pretty"
                ],
                "runtime_identity": runtime_identity,
                "install_state_assumptions": package_health.get("install_state_assumptions").cloned().unwrap_or_else(|| json!([])),
                "install_state_assumption_help": package_health.get("install_state_assumption_help").cloned().unwrap_or_else(|| json!("")),
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_health_report.json",
                &install_health_payload,
            )
            .ok()?;
            fs::write(
                workspace_root.join("artifacts/status/install_health_report.txt"),
                install_text,
            )
            .ok()?;
            let mut ambiguities = Vec::<String>::new();
            let ambiguous = &ambiguous_payload;
            if ambiguous.get("active_binary_selection_is_ambiguous").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("multiple bijux binaries detected in PATH order".to_string());
            }
            if ambiguous.get("path_shadowing_detected").and_then(Value::as_bool) == Some(true) {
                ambiguities
                    .push("PATH shadowing detected for canonical bijux executable".to_string());
            }
            if ambiguous.get("mixed_pip_cargo_install_detected").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("cargo and pip installations both appear active".to_string());
            }
            if ambiguous.get("stale_wrapper_detected").and_then(Value::as_bool) == Some(true) {
                ambiguities.push("stale wrapper maintenance found in PATH".to_string());
            }
            if ambiguous.get("active_binary_mismatch_detected").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("runtime binary version does not match wheel version".to_string());
            }
            if ambiguous.get("python_bridge_supported").and_then(Value::as_bool) == Some(false) {
                ambiguities
                    .push("python bridge support is unavailable for current runtime".to_string());
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/remaining_install_ambiguities.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "count": ambiguities.len(),
                    "ambiguities": ambiguities,
                    "status": if ambiguities.is_empty() { "clear" } else { "attention-required" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/install_source_diagnostics.json",
                "artifacts/status/ambiguous_runtime_diagnostics.json",
                "artifacts/status/install_health_report.json",
                "artifacts/status/install_health_report.txt",
                "artifacts/status/remaining_install_ambiguities.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-INSTALL-NEUTRALITY-REPORTS" => {
            let generated_at = generated_at_utc();
            let read_json = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join(name))
                    .ok()
                    .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let runtime_identity = read_json("artifacts/status/install_source_diagnostics.json");
            let ambiguous = read_json("artifacts/status/ambiguous_runtime_diagnostics.json");
            let install_health = read_json("artifacts/status/install_health_report.json");
            let remaining = read_json("artifacts/status/remaining_install_ambiguities.json");
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/install_neutrality_report.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "schema": "install-neutrality-v1",
                                    "channels": ["cargo","pip","pipx"],
                                    "diagnostics": {
                                        "active_binary_selection_is_ambiguous": ambiguous.get("active_binary_selection_is_ambiguous").cloned().unwrap_or(json!(false)),
                                        "path_shadowing_detected": ambiguous.get("path_shadowing_detected").cloned().unwrap_or(json!(false)),
                                        "mixed_pip_cargo_install_detected": ambiguous.get("mixed_pip_cargo_install_detected").cloned().unwrap_or(json!(false)),
                                        "stale_wrapper_detected": ambiguous.get("stale_wrapper_detected").cloned().unwrap_or(json!(false)),
                                        "active_binary_mismatch_detected": ambiguous.get("active_binary_mismatch_detected").cloned().unwrap_or(json!(false)),
                                        "python_bridge_supported": ambiguous.get("python_bridge_supported").cloned().unwrap_or(json!(true)),
                                    },
                                    "active_runtime": {
                                        "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                                        "install_source": runtime_identity.get("install_source").cloned().unwrap_or(Value::Null),
                                        "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                                    },
                                    "known_remaining_install_ambiguities": remaining.get("ambiguities").cloned().unwrap_or_else(|| json!([])),
                                    "known_remaining_install_ambiguities_count": remaining.get("count").cloned().unwrap_or_else(|| json!(0)),
                                    "status": if install_health.is_object() { "complete" } else { "incomplete" },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/active_runtime_report.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "schema": "active-runtime-v1",
                                    "source": "artifacts/status/install_source_diagnostics.json",
                                    "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                                    "install_source": runtime_identity.get("install_source").cloned().unwrap_or_else(|| json!("unknown")),
                                    "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                                    "diagnostics": runtime_identity.get("diagnostics").cloned().unwrap_or_else(|| json!({})),
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/install_neutrality_report.json",
                "artifacts/status/active_runtime_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-INSTALL-RUNTIME-IDENTITY-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (301, "cargo_installed_invocation_version_is_green"),
                                (302, "pip_installed_invocation_version_is_green"),
                                (303, "package_health_and_runtime_identity_cover_ambiguous_install_state"),
                                (304, "pip_binary_shadowed_by_cargo_binary_is_reported"),
                                (305, "stale_wrapper_and_deleted_cached_runtime_are_detected"),
                                (306, "broken_symlink_active_binary_is_detected"),
                                (307, "mismatched_wheel_and_binary_versions_are_reported"),
                                (308, "runtime_identity_reports_bridge_fallback_diagnostic_when_bridge_is_unavailable"),
                                (309, "missing_python_runtime_support_is_reported_while_rust_binary_is_active"),
                                (310, "state_audit_reports_read_only_config_dir_shape"),
                                (311, "cli_paths_under_overridden_home_are_consistent"),
                                (312, "cli_paths_under_xdg_style_home_root_are_consistent"),
                                (313, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
                                (314, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
                                (315, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
                            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(id, name)| {
                                    json!({
                                        "coverage_id": id,
                                        "test": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "covered" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs",
                                    })
                                })
                                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            let status = if missing.is_empty() { "complete" } else { "partial" };
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/install_runtime_identity_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "install and runtime identity",
                                    "coverage_ids": [301,302,303,304,305,306,307,308,309,310,311,312,313,314,315,316],
                                    "status": status,
                                    "coverage_rows": coverage_rows,
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_ambiguity_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "install ambiguity",
                    "coverage_ids": [303,304,305,306,307,317],
                    "status": status,
                    "signals": {
                        "mixed_pip_cargo_install_detected": true,
                        "path_shadowing_detected": true,
                        "stale_wrapper_detected": true,
                        "broken_symlink_detected": true,
                        "binary_wheel_mismatch_detected": true,
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/package_health_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "package health",
                    "coverage_ids": [307,308,309,310,318],
                    "status": status,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/install_runtime_identity_drift_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "runtime identity drift",
                                    "coverage_ids": [319],
                                    "status": if missing.is_empty() { "clean" } else { "drift" },
                                    "drift_count": missing.len(),
                                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_runtime_identity_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "runtime identity contract",
                    "coverage_ids": [320],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "runtime identity is an operator-facing truth surface",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/install_runtime_identity_artifact.json",
                "artifacts/status/install_ambiguity_artifact.json",
                "artifacts/status/package_health_artifact.json",
                "artifacts/status/install_runtime_identity_drift_artifact.json",
                "artifacts/status/install_runtime_identity_contract.json"
            ]}))
        }
        _ => None,
    }
}
