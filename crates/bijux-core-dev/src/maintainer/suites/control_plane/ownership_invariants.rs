#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;
use crate::schema::command_registry::command_registry;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-MAINTAINER-INVARIANTS-REPORTS" => {
            let fixture = workspace_root.join(
                "crates/bijux-core-dev/tests/maintainer/data/fixtures/routing/maintainer_subcommands.txt",
            );
            let bin_main = workspace_root.join("crates/bijux-cli/src/bin/bijux.rs");
            let runtime_entrypoint = workspace_root.join("crates/bijux-cli/src/bootstrap/run.rs");
            let runtime_dispatch =
                workspace_root.join("crates/bijux-cli/src/interface/cli/dispatch.rs");
            let lib_source = workspace_root.join("crates/bijux-core-dev/src/maintainer/lib.rs");
            let command_registry =
                workspace_root.join("crates/bijux-core-dev/src/maintainer/schema/command_registry.rs");
            let maintainer_root =
                workspace_root.join("crates/bijux-core-dev/src/maintainer/cli/routes/root.rs");
            let commands: Vec<Vec<String>> = fs::read_to_string(fixture)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| line.split(' ').map(ToString::to_string).collect::<Vec<_>>())
                .collect();
            let unique = commands.iter().collect::<BTreeSet<_>>().len() == commands.len();
            let mut help_stable = true;
            let mut json_parseable = true;
            let mut text_non_empty = true;
            let mut failures = Vec::<String>::new();
            for command in &commands {
                let mut json_args: Vec<String> = command.to_vec();
                json_args.extend([
                    "--format".to_string(),
                    "json".to_string(),
                    "--no-pretty".to_string(),
                ]);
                let json_refs = json_args.iter().map(String::as_str).collect::<Vec<_>>();
                match run_bijux_json(workspace_root, &json_refs) {
                    Ok(payload) => {
                        if !payload.is_object() {
                            json_parseable = false;
                            failures
                                .push(format!("json payload not object: {}", json_args.join(" ")));
                        }
                    }
                    Err(_) => {
                        json_parseable = false;
                        failures.push(format!("json command failed: {}", json_args.join(" ")));
                    }
                }
                let mut text_args: Vec<String> = command.to_vec();
                text_args.extend(["--format".to_string(), "text".to_string()]);
                let text_refs = text_args.iter().map(String::as_str).collect::<Vec<_>>();
                match run_bijux_text(workspace_root, &text_refs) {
                    Ok(text) => {
                        if text.trim().is_empty() {
                            text_non_empty = false;
                            failures.push(format!("text output invalid: {}", text_args.join(" ")));
                        }
                    }
                    Err(_) => {
                        text_non_empty = false;
                        failures.push(format!("text output invalid: {}", text_args.join(" ")));
                    }
                }
                let mut help_args: Vec<String> = command.to_vec();
                help_args.push("--help".to_string());
                let help_refs = help_args.iter().map(String::as_str).collect::<Vec<_>>();
                let first = run_bijux_text(workspace_root, &help_refs);
                let second = run_bijux_text(workspace_root, &help_refs);
                if first.is_err() || second.is_err() || first.ok() != second.ok() {
                    help_stable = false;
                    failures.push(format!("help output drift: {}", help_args.join(" ")));
                }
            }
            let status_base = run_bijux_json(workspace_root, &["status"]);
            let status_quiet = run_bijux_json(workspace_root, &["status", "--quiet"]);
            let quiet_exit_same = status_base.is_ok() == status_quiet.is_ok();

            let bin_source = fs::read_to_string(bin_main).unwrap_or_default();
            let runtime_entry_source = fs::read_to_string(runtime_entrypoint).unwrap_or_default();
            let runtime_dispatch_source = fs::read_to_string(runtime_dispatch).unwrap_or_default();
            let lib_text = fs::read_to_string(lib_source).unwrap_or_default();
            let command_registry_source = fs::read_to_string(command_registry).unwrap_or_default();
            let maintainer_root_source = fs::read_to_string(maintainer_root).unwrap_or_default();
            let checks = json!({
                "canonical_runtime_entrypoint_exists": runtime_entry_source.contains("run_app(&argv)"),
                "runtime_bootstrap_emits_results": runtime_entry_source.contains("emit_run_result(&result)"),
                "runtime_dispatch_stays_runtime_only": !runtime_dispatch_source.contains("bijux-dev-cli"),
                "runtime_law_not_in_control_plane": lib_text.contains("Runtime command law remains in runtime crates"),
                "command_registry_single_source": command_registry_source.contains("MAINTAINER_COMMAND_NAMESPACE"),
                "command_metadata_inspectable": command_registry_source.contains("pub fn command_registry()"),
                "command_names_stable": unique,
                "help_outputs_stable": help_stable,
                "json_outputs_parseable": json_parseable,
                "text_outputs_non_empty": text_non_empty,
                "quiet_mode_exit_semantics_stable": quiet_exit_same,
                "bin_entrypoint_is_thin_dispatcher": !bin_source.contains("bijux-dev-cli"),
                "maintainer_route_builders_live_in_control_plane":
                    maintainer_root_source.contains("dev_routes::build_report_from_query")
                    && maintainer_root_source.contains("dev_registry::build_report_from_query"),
            });
            let drift_checks: Vec<String> = checks
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let report = json!({
                "generator": "bijux-dev-cli",
                "scope": "maintainer control-plane invariants",
                "status": if drift_checks.is_empty() { "complete" } else { "partial" },
                "checks": checks,
                "failures": failures,
            });
            let drift = json!({
                "generator": "bijux-dev-cli",
                "scope": "maintainer control-plane invariants drift",
                "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                "drift_count": drift_checks.len(),
                "drift_checks": drift_checks,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/maintainer_invariants_artifact.json",
                &report,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/maintainer_invariants_drift_artifact.json",
                &drift,
            )
            .ok()?;
            Some(json!({
                "status":"ok",
                "contract_id":contract_id,
                "implementation":"rust",
                "outputs":[
                    "artifacts/status/maintainer_invariants_artifact.json",
                    "artifacts/status/maintainer_invariants_drift_artifact.json"
                ]
            }))
        }
        "STATUS-CONTRACT-GENERATE-MAINTAINER-ROUTE-REGISTRY-OWNERSHIP-DIFF" => {
            let runtime_dispatch =
                workspace_root.join("crates/bijux-cli/src/interface/cli/dispatch.rs");
            let runtime_routing = workspace_root.join("crates/bijux-cli/src/routing/mod.rs");
            let maintainer_routes =
                workspace_root.join("crates/bijux-core-dev/src/maintainer/reports/runtime_surface/routes.rs");
            let maintainer_registry =
                workspace_root.join("crates/bijux-core-dev/src/maintainer/reports/runtime_surface/registry.rs");
            let inventory = workspace_root
                .join("crates/bijux-cli/src/features/diagnostics/routing_inventory.rs");
            let has = |path: &Path, token: &str| -> bool {
                fs::read_to_string(path).map(|text| text.contains(token)).unwrap_or(false)
            };
            let boundaries = json!({
                "runtime_dispatch_mentions_maintainer_surface": has(&runtime_dispatch, "bijux-dev-cli"),
                "runtime_routing_builds_report_payloads":
                    has(&runtime_routing, "build_report(") || has(&runtime_routing, "build_report_from_query("),
                "maintainer_owns_routes_presentation":
                    has(&maintainer_routes, "pub fn build_report_from_query"),
                "maintainer_owns_registry_presentation":
                    has(&maintainer_registry, "pub fn build_report_from_query"),
                "routing_exposes_read_only_route_inventory": has(&inventory, "pub fn route_inventory"),
                "routing_exposes_read_only_registry_inventory": has(&inventory, "pub fn registry_inventory"),
            });
            let summary = json!({
                "ownership_boundary_is_clean":
                    boundaries["runtime_dispatch_mentions_maintainer_surface"] == false
                    && boundaries["runtime_routing_builds_report_payloads"] == false
                    && boundaries["maintainer_owns_routes_presentation"] == true
                    && boundaries["maintainer_owns_registry_presentation"] == true
                    && boundaries["routing_exposes_read_only_route_inventory"] == true
                    && boundaries["routing_exposes_read_only_registry_inventory"] == true,
            });
            let payload = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "route-registry ownership boundaries",
                "boundaries": boundaries,
                "summary": summary,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/maintainer_route_registry_ownership_diff.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/maintainer_route_registry_ownership_diff.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-MAINTAINER-DIAGNOSTICS-SOURCE-MAP" => {
            let payload = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "maintainer diagnostics source map",
                "commands": [
                    {
                        "command": "bijux-dev-cli runtime-identity",
                        "presentation_owner": "bijux-dev-cli",
                        "runtime_data_sources": [
                            "bijux-cli::install::install_health_report",
                            "bijux-cli::install::cargo_install_strategy",
                            "bijux-cli::install::pip_install_strategy",
                        ],
                    },
                    {
                        "command": "bijux-dev-cli package-health",
                        "presentation_owner": "bijux-dev-cli",
                        "runtime_data_sources": ["artifacts/status/current_rust_state.json"],
                    },
                    {
                        "command": "bijux-dev-cli state-audit",
                        "presentation_owner": "bijux-dev-cli",
                        "runtime_data_sources": [
                            "bijux-cli::state_path_status",
                            "bijux-cli::state_diagnostics",
                        ],
                    },
                    {
                        "command": "bijux-dev-cli state-doctor",
                        "presentation_owner": "bijux-dev-cli",
                        "runtime_data_sources": ["bijux-cli::state_diagnostics"],
                    },
                ],
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/maintainer_diagnostics_source_map.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/maintainer_diagnostics_source_map.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-MAINTAINER-INTERFACE-BRIDGE-REPORT" => {
            let query_files = [
                (
                    "routing_inventory",
                    workspace_root
                        .join("crates/bijux-cli/src/features/diagnostics/routing_inventory.rs"),
                ),
                ("contracts_query", workspace_root.join("crates/bijux-cli/src/contracts/query.rs")),
                (
                    "install_query",
                    workspace_root.join("crates/bijux-cli/src/features/install/query.rs"),
                ),
                (
                    "parity_status_query",
                    workspace_root
                        .join("crates/bijux-cli/src/features/diagnostics/parity_status.rs"),
                ),
                (
                    "state_diagnostics_query",
                    workspace_root
                        .join("crates/bijux-cli/src/features/diagnostics/state_diagnostics.rs"),
                ),
            ];
            let interfaces: Vec<Value> = query_files
                .into_iter()
                .map(|(name, path)| {
                    let text = fs::read_to_string(&path).unwrap_or_default();
                    json!({
                        "name": name,
                        "path": rel(&path, workspace_root),
                        "public_structs": text.matches("pub struct ").count(),
                        "public_functions": text.matches("pub fn ").count(),
                        "contains_json_assembly": text.contains("serde_json::json!"),
                        "contains_terminal_rendering": text.contains("println!")
                            || text.contains("eprintln!")
                            || text.contains("render_value("),
                    })
                })
                .collect();
            let report = json!({
                "scope": "runtime query interface bridge",
                "status": "ok",
                "interfaces": interfaces,
                "rules": [
                    "interfaces are read-only",
                    "interfaces are structured-data only",
                    "interfaces do not render text",
                        "interfaces bridge runtime data to bijux-dev-cli report assembly",
                    ],
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/maintainer_interface_bridge_report.json",
                &report,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/maintainer_interface_bridge_report.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-MAINTAINER-OWNERSHIP-REPORT" => {
            let command_rows = command_registry()
                .iter()
                .map(|entry| {
                    json!({
                        "command": entry.command.as_str(),
                        "group": entry.group.as_str(),
                        "visible": entry.visible,
                    })
                })
                .collect::<Vec<_>>();
            let visible = command_rows
                .iter()
                .filter(|row| row.get("visible").and_then(Value::as_bool) == Some(true))
                .count();
            let groups: BTreeSet<String> = command_rows
                .iter()
                .filter_map(|row| row.get("group").and_then(Value::as_str).map(ToString::to_string))
                .collect();
            let report = json!({
                "namespace": "bijux-dev-cli",
                "owner": "bijux-dev-cli",
                "commands": command_rows
                    .iter()
                    .map(|row| {
                        let mut obj = row.as_object().cloned().unwrap_or_default();
                        obj.insert("owner".to_string(), Value::String("bijux-dev-cli".to_string()));
                        Value::Object(obj)
                    })
                    .collect::<Vec<_>>(),
                "summary": {
                    "total": command_rows.len(),
                    "visible": visible,
                    "internal": command_rows.len().saturating_sub(visible),
                    "groups": groups,
                },
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/maintainer_ownership_report.json",
                &report,
            )
            .ok()?;
            let mut lines = vec![
                "Maintainer ownership report".to_string(),
                "owner: bijux-dev-cli".to_string(),
                "namespace: bijux-dev-cli".to_string(),
                String::new(),
            ];
            for row in &command_rows {
                let command = row.get("command").and_then(Value::as_str).unwrap_or("");
                let group = row.get("group").and_then(Value::as_str).unwrap_or("");
                let visibility = if row.get("visible").and_then(Value::as_bool) == Some(true) {
                    "visible"
                } else {
                    "internal"
                };
                lines.push(format!("- {command} [{group}, {visibility}]"));
            }
            fs::write(
                workspace_root.join("artifacts/status/maintainer_ownership_report.txt"),
                lines.join("\n") + "\n",
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/maintainer_ownership_report.json","artifacts/status/maintainer_ownership_report.txt"]}),
            )
        }
        _ => None,
    }
}
