#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    let report_writers: [(&str, &str, [&str; 4]); 5] = [
        (
            "artifacts/status/repo_health_report.json",
            "dev cli repo health",
            ["dev", "cli", "repo", "health"],
        ),
        (
            "artifacts/status/repo_drift_report.json",
            "dev cli repo drift",
            ["dev", "cli", "repo", "drift"],
        ),
        (
            "artifacts/status/repo_inventories_report.json",
            "dev cli repo inventories",
            ["dev", "cli", "repo", "inventories"],
        ),
        (
            "artifacts/status/repo_generated_report.json",
            "dev cli repo generated",
            ["dev", "cli", "repo", "generated"],
        ),
        (
            "artifacts/status/repo_stale_report.json",
            "dev cli repo stale",
            ["dev", "cli", "repo", "stale"],
        ),
    ];
    match contract_id {
        "STATUS-CONTRACT-GENERATE-REPO-HEALTH-REPORTS" => {
            let mut outputs = Vec::<String>::new();
            for (artifact, _label, cmd) in report_writers {
                let payload = match run_bijux_json(workspace_root, &cmd) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                if let Err(err) = write_status_artifact_json(workspace_root, artifact, &payload) {
                    return Some(
                        json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                    );
                }
                outputs.push(artifact.to_string());
            }
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":outputs}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-EVIDENCE-REPORTS" => {
            let rows = [
                (
                    "artifacts/status/dev_cli_evidence_list_report.json",
                    ["dev", "cli", "evidence", "list"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_audit_report.json",
                    ["dev", "cli", "evidence", "audit"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_stale_report.json",
                    ["dev", "cli", "evidence", "stale"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_matrix_report.json",
                    ["dev", "cli", "evidence", "matrix"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_website_export_report.json",
                    ["dev", "cli", "evidence", "website-export"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_ci_export_report.json",
                    ["dev", "cli", "evidence", "ci-export"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_release_export_report.json",
                    ["dev", "cli", "evidence", "release-export"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_command_map_report.json",
                    ["dev", "cli", "evidence", "command-map"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_parity_map_report.json",
                    ["dev", "cli", "evidence", "parity-map"],
                ),
            ];
            let mut outputs = Vec::<String>::new();
            for (artifact, cmd) in rows {
                let payload = match run_bijux_json(workspace_root, &cmd) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                if let Err(err) = write_status_artifact_json(workspace_root, artifact, &payload) {
                    return Some(
                        json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                    );
                }
                outputs.push(artifact.to_string());
            }
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":outputs}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-RELEASE-REPORTS" => {
            let rows = [
                (
                    "artifacts/status/dev_cli_release_status_report.json",
                    ["dev", "cli", "release", "status"],
                ),
                (
                    "artifacts/status/dev_cli_release_evidence_report.json",
                    ["dev", "cli", "release", "evidence"],
                ),
                (
                    "artifacts/status/dev_cli_release_readiness_report.json",
                    ["dev", "cli", "release", "readiness"],
                ),
                (
                    "artifacts/status/dev_cli_release_diff_report.json",
                    ["dev", "cli", "release", "diff"],
                ),
                (
                    "artifacts/status/dev_cli_release_gaps_report.json",
                    ["dev", "cli", "release", "gaps"],
                ),
                (
                    "artifacts/status/dev_cli_release_summary_report.json",
                    ["dev", "cli", "release", "summary"],
                ),
                (
                    "artifacts/status/dev_cli_release_manifest_report.json",
                    ["dev", "cli", "release", "manifest"],
                ),
                (
                    "artifacts/status/dev_cli_release_notes_report.json",
                    ["dev", "cli", "release", "notes"],
                ),
                (
                    "artifacts/status/dev_cli_release_behavior_changes_report.json",
                    ["dev", "cli", "release", "behavior-changes"],
                ),
                (
                    "artifacts/status/dev_cli_release_intentional_differences_report.json",
                    ["dev", "cli", "release", "intentional-differences"],
                ),
                (
                    "artifacts/status/dev_cli_release_unresolved_gaps_report.json",
                    ["dev", "cli", "release", "unresolved-gaps"],
                ),
                (
                    "artifacts/status/dev_cli_release_compatibility_leftovers_report.json",
                    ["dev", "cli", "release", "compatibility-leftovers"],
                ),
            ];
            let mut outputs = Vec::<String>::new();
            for (artifact, cmd) in rows {
                let payload = match run_bijux_json(workspace_root, &cmd) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                if let Err(err) = write_status_artifact_json(workspace_root, artifact, &payload) {
                    return Some(
                        json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                    );
                }
                outputs.push(artifact.to_string());
            }
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":outputs}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-COCKPIT-REPORTS" => {
            let rows = [
                ("dev_cli_status_report.json", ["dev", "cli", "status"]),
                ("dev_cli_dashboard_report.json", ["dev", "cli", "dashboard"]),
                ("dev_cli_quickcheck_report.json", ["dev", "cli", "quickcheck"]),
                ("dev_cli_truth_report.json", ["dev", "cli", "truth"]),
                ("dev_cli_blockers_report.json", ["dev", "cli", "blockers"]),
                ("dev_cli_next_report.json", ["dev", "cli", "next"]),
            ];
            let mut payloads = BTreeMap::<String, Value>::new();
            let mut text_heads = BTreeMap::<String, String>::new();
            let mut outputs = Vec::<String>::new();
            for (artifact, cmd) in rows {
                let payload = match run_bijux_json(workspace_root, &cmd) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                let artifact_path = format!("artifacts/status/{artifact}");
                if let Err(err) =
                    write_status_artifact_json(workspace_root, &artifact_path, &payload)
                {
                    return Some(
                        json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                    );
                }
                let text = match run_bijux_text(workspace_root, &cmd) {
                    Ok(text) => text,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                text_heads
                    .insert(cmd.join(" "), text.lines().take(3).collect::<Vec<_>>().join("\n"));
                payloads.insert(artifact.to_string(), payload);
                outputs.push(artifact_path);
            }
            if let Err(err) = write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_cockpit_text_heads.json",
                &json!(text_heads),
            ) {
                return Some(
                    json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                );
            }
            let status_summary = payloads
                .get("dev_cli_status_report.json")
                .and_then(|v| v.get("status_report"))
                .and_then(|v| v.get("summary"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let truth_payload = payloads
                .get("dev_cli_truth_report.json")
                .and_then(|v| v.get("truth"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let truth_done = truth_payload
                .get("done")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let truth_missing = truth_payload
                .get("missing")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let truth_partial = truth_payload
                .get("partial")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let truth_intentional = truth_payload
                .get("intentional_differences")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let blockers = payloads
                .get("dev_cli_blockers_report.json")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let unresolved: BTreeSet<String> = payloads
                .get("dev_cli_status_report.json")
                .and_then(|v| v.get("status_report"))
                .and_then(|v| v.get("commands"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("complete"))
                .filter_map(|row| {
                    row.get("command").and_then(Value::as_str).map(ToString::to_string)
                })
                .collect();
            let blocker_commands: Vec<String> = blockers
                .into_iter()
                .filter_map(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                        .or_else(|| row.as_str().map(ToString::to_string))
                })
                .collect();
            let blocker_subset_ok =
                blocker_commands.iter().all(|command| unresolved.contains(command));
            let next_policy = payloads
                .get("dev_cli_next_report.json")
                .and_then(|v| v.get("next"))
                .and_then(|v| v.get("minimalism"))
                .and_then(|v| v.get("evidence_first_policy"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let next_derived_ok = next_policy
                .get("manual_curated_priority_lists_allowed")
                .and_then(Value::as_bool)
                == Some(false)
                && next_policy.get("roadmap_requires_generated_artifacts").and_then(Value::as_bool)
                    == Some(true)
                && next_policy
                    .get("required_artifacts")
                    .and_then(Value::as_array)
                    .is_some_and(|items| !items.is_empty());
            let dashboard_status_match = payloads
                .get("dev_cli_dashboard_report.json")
                .and_then(|v| v.get("dashboard"))
                .and_then(|v| v.get("status"))
                .and_then(|v| v.get("summary"))
                == Some(&status_summary);
            let count_alignment_ok = status_summary.get("complete").and_then(Value::as_i64)
                == Some(truth_done)
                && status_summary.get("missing").and_then(Value::as_i64) == Some(truth_missing)
                && status_summary.get("partial").and_then(Value::as_i64).unwrap_or(0)
                    + status_summary.get("shim").and_then(Value::as_i64).unwrap_or(0)
                    == truth_partial + truth_intentional;
            let summary_checks = json!({
                "status_truth_count_alignment": count_alignment_ok,
                "blockers_subset_of_unresolved_status": blocker_subset_ok,
                "next_derived_from_generated_evidence_status": next_derived_ok,
                "dashboard_matches_standalone_status_summary": dashboard_status_match,
            });
            let summary_artifact = json!({
                "scope": "dev cli summary surface",
                "generator": "bijux-dev-cli",
                "checks": summary_checks,
                "status": if summary_checks.as_object().is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true))) { "complete" } else { "partial" },
            });
            let drift_checks: Vec<String> = summary_checks
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter(|(_, value)| value.as_bool() != Some(true))
                        .map(|(name, _)| name.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let drift_artifact = json!({
                "scope": "dev cli summary surface drift",
                "generator": "bijux-dev-cli",
                "drift_checks": drift_checks,
                "drift_count": drift_checks.len(),
                "status": if drift_checks.is_empty() { "clean" } else { "drift" },
            });
            if let Err(err) = write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_summary_surface_artifact.json",
                &summary_artifact,
            ) {
                return Some(
                    json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                );
            }
            if let Err(err) = write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_summary_surface_drift_artifact.json",
                &drift_artifact,
            ) {
                return Some(
                    json!({"status":"failed","contract_id":contract_id,"implementation":"rust","error":err}),
                );
            }
            outputs.push("artifacts/status/dev_cli_cockpit_text_heads.json".to_string());
            outputs.push("artifacts/status/dev_cli_summary_surface_artifact.json".to_string());
            outputs
                .push("artifacts/status/dev_cli_summary_surface_drift_artifact.json".to_string());
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":outputs}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-MAINTENANCE-MIGRATION-REPORTS" => {
            let remaining =
                run_bijux_json(workspace_root, &["dev", "cli", "maintenance", "remaining"]).ok()?;
            let migrated =
                run_bijux_json(workspace_root, &["dev", "cli", "maintenance", "migrated"]).ok()?;
            let diff = run_bijux_json(workspace_root, &["dev", "cli", "maintenance", "diff"]).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_maintenance_remaining_report.json",
                &remaining,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_maintenance_migrated_report.json",
                &migrated,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_maintenance_diff_report.json",
                &diff,
            )
            .ok()?;
            let mut ranking: Vec<Value> = migrated
                                .get("migrated")
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_default()
                                .into_iter()
                                .map(|row| {
                                    json!({
                                        "source": row.get("from").cloned().unwrap_or(Value::Null),
                                        "replacement": row.get("to").cloned().unwrap_or(Value::Null),
                                        "maintainer_value_rank": row.get("maintainer_value_rank").cloned().unwrap_or_else(|| json!(0)),
                                    })
                                })
                                .collect();
            ranking.sort_by(|left, right| {
                let l = left.get("maintainer_value_rank").and_then(Value::as_i64).unwrap_or(0);
                let r = right.get("maintainer_value_rank").and_then(Value::as_i64).unwrap_or(0);
                r.cmp(&l)
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_maintenance_value_ranking.json",
                &json!({"ranking": ranking}),
            )
            .ok()?;
            let make_targets = remaining
                .get("make_targets")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_make_target_inventory.json",
                &json!({
                    "make_targets": make_targets,
                    "count": make_targets.len(),
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/dev_cli_maintenance_remaining_report.json",
                "artifacts/status/dev_cli_maintenance_migrated_report.json",
                "artifacts/status/dev_cli_maintenance_diff_report.json",
                "artifacts/status/dev_cli_maintenance_value_ranking.json",
                "artifacts/status/dev_cli_make_target_inventory.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-REPO-DOCS-MAINTENANCE-CRATE-HEALTH-REPORTS" => {
            let repo = run_bijux_json(workspace_root, &["dev", "cli", "repo", "health"]).ok()?;
            let docs = run_bijux_json(workspace_root, &["dev", "cli", "docs-audit"]).ok()?;
            let maintenance = run_bijux_json(workspace_root, &["dev", "cli", "maintenance-audit"]).ok()?;
            let crate_health =
                run_bijux_json(workspace_root, &["dev", "cli", "crate-health"]).ok()?;
            let checks = json!({
                "repo_health_payload_present": repo.get("repo_health").is_some_and(Value::is_object),
                "docs_payload_present": docs.get("docs").is_some_and(Value::is_array),
                "maintenance_payload_present": maintenance.get("maintenance").is_some_and(Value::is_array),
                "crate_metrics_payload_present": crate_health.get("crate_metrics").is_some_and(Value::is_object),
                "docs_audit_summary_present": docs.get("docs_audit").is_some_and(Value::is_object),
                "maintenance_audit_remaining_signal_present": maintenance.get("remaining_legacy_only_behaviors").is_some(),
                "crate_health_dependency_edges_present": crate_health.get("dependency_edges").is_some_and(Value::is_array),
                "crate_health_public_api_inventory_present": crate_health.get("public_api_by_crate").is_some_and(Value::is_object),
                "repo_health_stale_generated_signal_present":
                    repo.get("repo_health").and_then(|v| v.get("generated")).and_then(|v| v.get("stale_generated_artifacts")).is_some_and(Value::is_array)
                    || repo.get("repo_health").and_then(|v| v.get("stale")).and_then(|v| v.get("stale_generated_artifacts")).is_some_and(Value::is_array),
            });
            let drift_checks: Vec<String> = checks
                .as_object()
                .map(|o| {
                    o.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repo_docs_maintenance_crate_health_artifact.json",
                &json!({
                    "scope": "repo/docs/maintenance/crate-health truth",
                    "generator": "bijux-dev-cli",
                    "checks": checks,
                    "status": if drift_checks.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repo_docs_maintenance_crate_health_drift_artifact.json",
                &json!({
                    "scope": "repo/docs/maintenance/crate-health drift",
                    "generator": "bijux-dev-cli",
                    "drift_checks": drift_checks,
                    "drift_count": drift_checks.len(),
                    "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/repo_docs_maintenance_crate_health_artifact.json",
                "artifacts/status/repo_docs_maintenance_crate_health_drift_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-ROUTE-REGISTRY-ENV-CONTRACTS-REPORTS" => {
            let routes = run_bijux_json(workspace_root, &["dev", "cli", "routes"]).ok()?;
            let registry = run_bijux_json(workspace_root, &["dev", "cli", "registry"]).ok()?;
            let env = run_bijux_json(workspace_root, &["dev", "cli", "env"]).ok()?;
            let contracts = run_bijux_json(workspace_root, &["dev", "cli", "contracts"]).ok()?;
            let inspect = run_bijux_json(workspace_root, &["inspect"]).ok()?;
            let route_roots: BTreeSet<String> = routes
                .get("routes")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|row| {
                    row.get("segments")
                        .and_then(Value::as_array)
                        .and_then(|s| s.first())
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
                .collect();
            let inspect_roots: BTreeSet<String> = inspect
                .get("route_sources")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|row| {
                    row.get("segments")
                        .and_then(Value::as_array)
                        .and_then(|s| s.first())
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
                .collect();
            let checks = json!({
                "routes_payload_present": routes.get("routes").is_some_and(Value::is_array),
                "registry_payload_present": registry.get("registry").is_some_and(Value::is_array),
                "env_payload_present": env.get("source_precedence").is_some_and(Value::is_array),
                "contracts_payload_present": contracts.get("contracts").is_some_and(|v| v.is_array() || v.is_object()),
                "routes_agree_with_inspect_roots": route_roots.is_subset(&inspect_roots),
                "registry_has_ownership_metadata": registry.get("ownership").is_some_and(Value::is_object),
                "env_has_active_and_precedence": env.get("active").is_some_and(Value::is_object) && env.get("source_precedence").is_some_and(Value::is_array),
                "contracts_has_schema_runtime_versions": contracts.get("schema_version").is_some_and(Value::is_string) && contracts.get("runtime_version").is_some_and(Value::is_string),
            });
            let drift_checks: Vec<String> = checks
                .as_object()
                .map(|o| {
                    o.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_registry_env_contracts_artifact.json",
                &json!({
                    "scope": "routes/registry/env/contracts truth",
                    "generator": "bijux-dev-cli",
                    "checks": checks,
                    "status": if drift_checks.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_registry_env_contracts_drift_artifact.json",
                &json!({
                    "scope": "routes/registry/env/contracts drift",
                    "generator": "bijux-dev-cli",
                    "drift_checks": drift_checks,
                    "drift_count": drift_checks.len(),
                    "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/route_registry_env_contracts_artifact.json",
                "artifacts/status/route_registry_env_contracts_drift_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-RUSTDOC-REPORTS" => {
            let audit = run_bijux_json(workspace_root, &["dev", "cli", "rustdoc", "audit"]).ok()?;
            let coverage =
                run_bijux_json(workspace_root, &["dev", "cli", "rustdoc", "coverage"]).ok()?;
            let audit_text =
                run_bijux_text(workspace_root, &["dev", "cli", "rustdoc", "audit"]).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/rustdoc_audit_report.json",
                &audit,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/rustdoc_public_api_coverage_report.json",
                &coverage,
            )
            .ok()?;
            fs::write(workspace_root.join("artifacts/status/rustdoc_audit_report.txt"), audit_text)
                .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/rustdoc_audit_report.json",
                "artifacts/status/rustdoc_public_api_coverage_report.json",
                "artifacts/status/rustdoc_audit_report.txt"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-RELEASE-TRUTH-BUNDLE" => {
            let commands = [
                ("status", ["dev", "cli", "release", "status"]),
                ("evidence", ["dev", "cli", "release", "evidence"]),
                ("readiness", ["dev", "cli", "release", "readiness"]),
                ("diff", ["dev", "cli", "release", "diff"]),
                ("gaps", ["dev", "cli", "release", "gaps"]),
                ("behavior_changes", ["dev", "cli", "release", "behavior-changes"]),
                ("intentional_differences", ["dev", "cli", "release", "intentional-differences"]),
                ("unresolved_gaps", ["dev", "cli", "release", "unresolved-gaps"]),
                ("compatibility_leftovers", ["dev", "cli", "release", "compatibility-leftovers"]),
            ];
            let mut reports = serde_json::Map::new();
            for (name, cmd) in commands {
                reports.insert(name.to_string(), run_bijux_json(workspace_root, &cmd).ok()?);
            }
            let gaps = reports.get("gaps").cloned().unwrap_or_else(|| json!({}));
            let unresolved =
                gaps.get("unresolved_gaps").and_then(Value::as_array).map_or(0, Vec::len);
            let missing =
                gaps.get("missing_evidence").and_then(Value::as_array).map_or(0, Vec::len);
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_release_truth_bundle.json",
                &json!({
                    "source": "dev cli release *",
                    "reports": reports,
                    "summary": {
                        "unresolved_gaps": unresolved,
                        "missing_evidence": missing,
                    }
                }),
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_release_truth_bundle.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-CONTROL-PLANE-BUNDLE" => {
            let commands = [
                "dev cli status",
                "dev cli parity",
                "dev cli runtime-identity",
                "dev cli state-audit",
                "dev cli package-health",
                "dev cli maintenance-audit",
                "dev cli rustdoc audit",
                "dev cli release status",
                "dev cli docs-audit",
                "dev cli crate-health",
            ];
            let mut payload = serde_json::Map::new();
            for command in commands {
                let argv: Vec<&str> = command.split(' ').collect();
                let row = run_bijux_json(workspace_root, &argv).ok()?;
                payload.insert(command.to_string(), json!({
                                    "top_level_keys": row.as_object().map(|obj| obj.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
                                    "payload": row,
                                }));
            }
            let ownership_path =
                workspace_root.join("artifacts/status/dev_cli_ownership_report.json");
            let ownership = fs::read_to_string(&ownership_path)
                .ok()
                .and_then(|text| serde_json::from_str::<Value>(&text).ok());
            let mut out = json!({
                "scope": "bijux-dev-cli control-plane bundle",
                "commands": payload,
            });
            if let Some(ownership) = ownership {
                out["ownership_report"] = ownership;
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_control_plane_bundle.json",
                &out,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_control_plane_bundle.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-MAINTAINER-REPORT-IO-MAP" => {
            let commands = ["dev cli env", "dev cli contracts", "dev cli parity", "dev cli status"];
            let mut input_map = BTreeMap::<&str, Vec<&str>>::new();
            input_map.insert(
                "dev cli env",
                vec!["process environment", "resolved config/history/plugins paths"],
            );
            input_map.insert(
                "dev cli contracts",
                vec!["static schema contract declarations", "runtime version"],
            );
            input_map.insert(
                "dev cli parity",
                vec!["artifacts/parity/*.json", "artifacts/parity/*.txt"],
            );
            input_map.insert(
                "dev cli status",
                vec![
                    "artifacts/status/*.json",
                    "artifacts/status/*.txt",
                    "artifacts/parity/rust_python_parity_report.json",
                    "dev-cli inventory payload",
                ],
            );
            let mut reports = Vec::<Value>::new();
            for command in commands {
                let argv: Vec<&str> = command.split(' ').collect();
                let payload = run_bijux_json(workspace_root, &argv).ok()?;
                let output_top_level_keys = payload
                    .as_object()
                    .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                reports.push(json!({
                    "command": command,
                    "inputs": input_map.get(command).cloned().unwrap_or_default(),
                    "output_top_level_keys": output_top_level_keys,
                    "output_kind": if payload.is_object() { "json-object" } else { "non-object" },
                }));
            }
            let payload = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "dev-cli maintainer report inputs vs outputs",
                "reports": reports,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_maintainer_report_io_map.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_maintainer_report_io_map.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-PARITY-CONSISTENCY-REPORTS" => {
            let parity_first = run_bijux_json(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let parity_second = run_bijux_json(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let status_payload = run_bijux_json(workspace_root, &["dev", "cli", "status"]).ok()?;
            let parity_text_first =
                run_bijux_text(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let parity_text_second =
                run_bijux_text(workspace_root, &["dev", "cli", "parity"]).ok()?;

            let valid_statuses = BTreeSet::from([
                "rust-complete",
                "rust-partial",
                "python-only",
                "intentionally-different",
            ]);
            let migration_rows = status_payload
                .get("command_migration")
                .and_then(|v| v.get("matrix"))
                .and_then(|v| v.get("commands"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let parity_rows = parity_first
                .get("command_matrix")
                .and_then(|v| v.get("commands"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let invalid_status_rows: Vec<String> = migration_rows
                .iter()
                .filter_map(|row| {
                    let status = row.get("status").and_then(Value::as_str)?;
                    if valid_statuses.contains(status) {
                        None
                    } else {
                        Some(row.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                    }
                })
                .collect();
            let partial_without_blocker: Vec<String> = migration_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("rust-partial"))
                .filter_map(|row| {
                    let blocker =
                        row.get("blocker").and_then(Value::as_str).unwrap_or("").trim().to_string();
                    let shim_alias = row
                        .get("shim_alias_dependency")
                        .and_then(Value::as_object)
                        .cloned()
                        .unwrap_or_default();
                    let has_shim_alias = shim_alias
                        .get("aliases")
                        .and_then(Value::as_array)
                        .is_some_and(|items| !items.is_empty())
                        || shim_alias
                            .get("shims")
                            .and_then(Value::as_array)
                            .is_some_and(|items| !items.is_empty());
                    let has_parity_mismatch = row
                        .get("parity_coverage")
                        .and_then(Value::as_object)
                        .is_some_and(|obj| obj.values().any(|v| v == &Value::Bool(false)));
                    if blocker.is_empty() && !has_shim_alias && !has_parity_mismatch {
                        Some(row.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let intentional_without_reason: Vec<String> = migration_rows
                .iter()
                .filter(|row| {
                    row.get("status").and_then(Value::as_str) == Some("intentionally-different")
                })
                .filter_map(|row| {
                    let reason = row.get("reason").and_then(Value::as_str).unwrap_or("").trim();
                    if reason.is_empty() {
                        Some(row.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let complete_without_evidence: Vec<String> = migration_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("rust-complete"))
                .filter_map(|row| {
                    if row.get("evidence_links").and_then(Value::as_array).is_none_or(Vec::is_empty)
                    {
                        Some(row.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let parity_commands: BTreeSet<String> = parity_rows
                .iter()
                .filter_map(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(ToString::to_string)
                })
                .collect();
            let migration_commands: BTreeSet<String> = migration_rows
                .iter()
                .filter_map(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(ToString::to_string)
                })
                .collect();
            let missing_from_migration: Vec<String> =
                parity_commands.difference(&migration_commands).cloned().collect();
            let parity_complete = parity_first
                .get("command_matrix")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("complete"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let migration_complete = status_payload
                .get("command_migration")
                .and_then(|v| v.get("matrix"))
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("rust-complete"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let consistency_checks = json!({
                "migration_rows_have_valid_status": invalid_status_rows.is_empty(),
                "partial_rows_have_blockers": partial_without_blocker.is_empty(),
                "intentional_rows_have_reasons": intentional_without_reason.is_empty(),
                "complete_rows_have_evidence_links": complete_without_evidence.is_empty(),
                "parity_commands_exist_in_migration_matrix": missing_from_migration.is_empty(),
                "parity_and_status_complete_counts_align": parity_complete == migration_complete,
                "parity_json_is_deterministic": parity_first == parity_second,
                "parity_text_is_deterministic": parity_text_first == parity_text_second,
            });
            let migration_truth_artifact = json!({
                "scope": "migration truth",
                "generator": "bijux-dev-cli",
                "rows_total": migration_rows.len(),
                "checks": {
                    "valid_status_rows": consistency_checks["migration_rows_have_valid_status"],
                    "partial_rows_with_blockers": consistency_checks["partial_rows_have_blockers"],
                    "intentional_rows_with_reasons": consistency_checks["intentional_rows_have_reasons"],
                    "complete_rows_with_evidence_links": consistency_checks["complete_rows_have_evidence_links"],
                },
                "status": if consistency_checks["migration_rows_have_valid_status"] == true
                    && consistency_checks["partial_rows_have_blockers"] == true
                    && consistency_checks["intentional_rows_have_reasons"] == true
                    && consistency_checks["complete_rows_have_evidence_links"] == true
                {
                    "complete"
                } else {
                    "partial"
                },
            });
            let parity_evidence_consistency_artifact = json!({
                "scope": "parity evidence consistency",
                "generator": "bijux-dev-cli",
                "checks": {
                    "parity_commands_exist_in_migration_matrix": consistency_checks["parity_commands_exist_in_migration_matrix"],
                    "parity_and_status_complete_counts_align": consistency_checks["parity_and_status_complete_counts_align"],
                    "parity_json_is_deterministic": consistency_checks["parity_json_is_deterministic"],
                    "parity_text_is_deterministic": consistency_checks["parity_text_is_deterministic"],
                },
                "status": if consistency_checks["parity_commands_exist_in_migration_matrix"] == true
                    && consistency_checks["parity_and_status_complete_counts_align"] == true
                    && consistency_checks["parity_json_is_deterministic"] == true
                    && consistency_checks["parity_text_is_deterministic"] == true
                {
                    "complete"
                } else {
                    "partial"
                },
            });
            let drift_checks: Vec<String> = consistency_checks
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let parity_drift_artifact = json!({
                "scope": "parity and migration drift",
                "generator": "bijux-dev-cli",
                "drift_checks": drift_checks,
                "drift_count": drift_checks.len(),
                "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                "details": {
                    "invalid_status_rows": invalid_status_rows,
                    "partial_without_blocker": partial_without_blocker,
                    "intentional_without_reason": intentional_without_reason,
                    "complete_without_evidence": complete_without_evidence,
                    "parity_missing_from_migration": missing_from_migration,
                },
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/migration_truth_artifact.json",
                &migration_truth_artifact,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/parity_evidence_consistency_artifact.json",
                &parity_evidence_consistency_artifact,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/parity_drift_artifact.json",
                &parity_drift_artifact,
            )
            .ok()?;
            Some(json!({
                "status":"ok",
                "contract_id":contract_id,
                "implementation":"rust",
                "outputs":[
                    "artifacts/status/migration_truth_artifact.json",
                    "artifacts/status/parity_evidence_consistency_artifact.json",
                    "artifacts/status/parity_drift_artifact.json"
                ]
            }))
        }
        _ => None,
    }
}
