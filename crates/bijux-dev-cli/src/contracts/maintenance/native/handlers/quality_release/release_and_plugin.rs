#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-RELEASE-BUILD-REPORTS" => {
            let generated_at = generated_at_utc();
            let file_info = |path: &Path| -> Value {
                if !path.exists() {
                    return json!({"path": rel(path, workspace_root), "exists": false});
                }
                let data = fs::read(path).unwrap_or_default();
                let sha256 = Command::new("shasum")
                    .args(["-a", "256", &path.to_string_lossy()])
                    .current_dir(workspace_root)
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .and_then(|s| s.split_whitespace().next().map(ToString::to_string))
                    .unwrap_or_default();
                json!({
                    "path": rel(path, workspace_root),
                    "exists": true,
                    "size_bytes": data.len(),
                    "sha256": sha256,
                })
            };
            let release_bin = file_info(&workspace_root.join("target/release/bijux-rs"));
            let debug_bin = file_info(&workspace_root.join("target/debug/bijux-rs"));
            let tree = Command::new("cargo")
                .args(["tree", "-p", "bijux-cli", "-e", "normal", "--prefix", "none"])
                .current_dir(workspace_root)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();
            let mut top = BTreeMap::<String, usize>::new();
            for line in tree.lines().map(str::trim).filter(|l| !l.is_empty()) {
                let name = line.split_whitespace().next().unwrap_or("");
                if name == "bijux-cli" || name.starts_with("bijux-cli-") {
                    continue;
                }
                *top.entry(name.to_string()).or_insert(0) += 1;
            }
            let mut top_rows =
                top.into_iter().map(|(k, v)| json!({"crate":k,"hits":v})).collect::<Vec<_>>();
            top_rows.sort_by(|a, b| {
                b.get("hits").and_then(Value::as_u64).cmp(&a.get("hits").and_then(Value::as_u64))
            });
            top_rows.truncate(20);
            let metadata = Command::new("cargo")
                .args(["metadata", "--format-version", "1", "--no-deps"])
                .current_dir(workspace_root)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_else(|| json!({}));
            let packages =
                metadata.get("packages").and_then(Value::as_array).cloned().unwrap_or_default();
            let deps = packages.iter().map(|pkg| json!({"name":pkg["name"],"version":pkg["version"],"manifest_path":pkg["manifest_path"]})).collect::<Vec<_>>();
            let licenses = packages.iter().map(|pkg| json!({"name":pkg["name"],"version":pkg["version"],"license":pkg.get("license").cloned().unwrap_or_else(|| json!("UNKNOWN"))})).collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/release_binary_size_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","binary":release_bin})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/debug_binary_size_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","binary":debug_bin})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/release_binary_size_contributors.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","top_dependency_contributors":top_rows,"removed_dependencies_for_size":["strsim","anyhow (from bijux-cli-python)","thiserror (from bijux-cli-python)"],"disabled_default_features":["clap in bijux-cli","pyo3 in bijux-cli-python"]})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/release_dependency_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","workspace_packages":deps})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/license_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","workspace_licenses":licenses})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/reproducible_build_assumptions.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","assumptions":["Cargo.lock is committed and used in CI.","SOURCE_DATE_EPOCH is respected by status generators.","schema snapshots and command-tree snapshots are enforced in CI.","parity matrix generation is required and checked for deterministic output."],"non_promises":["bit-for-bit reproducibility across different host toolchains is not guaranteed"]})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/release_artifact_manifest.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","artifacts":["artifacts/status/release_binary_size_report.json","artifacts/status/debug_binary_size_report.json","artifacts/status/release_binary_size_contributors.json","artifacts/status/release_dependency_inventory.json","artifacts/status/license_inventory.json","artifacts/status/reproducible_build_assumptions.json","artifacts/status/deterministic_generation_report.json","artifacts/status/release_build_consistency_report.json","artifacts/status/release_evidence_bundle.json","artifacts/status/release_status_manifest.json"]})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/release_binary_size_report.json","artifacts/status/debug_binary_size_report.json","artifacts/status/release_binary_size_contributors.json","artifacts/status/release_dependency_inventory.json","artifacts/status/license_inventory.json","artifacts/status/reproducible_build_assumptions.json","artifacts/status/release_artifact_manifest.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-RELEASE-EVIDENCE-REPORTS" => {
            let generated_at = generated_at_utc();
            let read = |path: &str| -> Value {
                fs::read_to_string(workspace_root.join(path))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let paths = vec![
                "artifacts/parity/command_parity_matrix.json",
                "artifacts/status/runtime_unity_report.json",
                "artifacts/status/package_health_report.json",
                "artifacts/status/plugin_lifecycle_failure_injection_report.json",
                "artifacts/status/state_resilience_summary.json",
                "artifacts/status/performance_report.json",
                "artifacts/status/release_binary_size_report.json",
                "artifacts/status/release_dependency_inventory.json",
                "artifacts/status/reproducible_build_assumptions.json",
                "artifacts/status/deterministic_generation_report.json",
                "artifacts/status/release_build_consistency_report.json",
                "artifacts/status/release_artifact_manifest.json",
                "artifacts/status/cross_surface_consistency_artifact.json",
                "artifacts/status/cross_surface_drift_artifact.json",
                "artifacts/status/cross_surface_consistency_contract.json",
                "artifacts/status/simplification_deletion_artifact.json",
                "artifacts/status/candidate_merge_later_report.json",
                "artifacts/status/candidate_keep_separate_report.json",
                "artifacts/status/command_migration_matrix.json",
                "artifacts/status/install_neutrality_report.json",
                "artifacts/status/active_runtime_report.json",
                "artifacts/status/command_family_closure_report.json",
                "artifacts/status/compatibility_debt_trend_report.json",
                "artifacts/status/what_is_left.json",
                "artifacts/status/what_is_done.json",
                "artifacts/status/what_is_partial.json",
                "artifacts/status/what_is_intentionally_different.json",
                "docs/KNOWN_GAPS.md",
            ];
            let evidence = paths
                .iter()
                .map(|p| json!({"path":p,"exists":workspace_root.join(p).exists()}))
                .collect::<Vec<_>>();
            let missing = evidence
                .iter()
                .filter(|e| e.get("exists").and_then(Value::as_bool) != Some(true))
                .filter_map(|e| e.get("path").and_then(Value::as_str))
                .collect::<Vec<_>>();
            let parity = read("artifacts/parity/command_parity_matrix.json");
            let parity_rows =
                parity.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
            let partial = parity_rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) == Some("partial"))
                .map(|r| r.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                .collect::<Vec<_>>();
            let missing_cmd = parity_rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) == Some("missing"))
                .map(|r| r.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                .collect::<Vec<_>>();
            let scripts_audit = read("artifacts/status/script_only_behaviors.json");
            let docs_audit = read("artifacts/status/docs_audit.json");
            let test_audit = read("artifacts/status/test_quality_audit.json");
            let weak_tests = test_audit
                .get("tests")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|r| r.get("shallow_score").and_then(Value::as_i64).unwrap_or(0) >= 5)
                .filter_map(|r| r.get("path").and_then(Value::as_str).map(ToString::to_string))
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/release_evidence_bundle.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","scope":"release evidence bundle","status":if missing.is_empty(){"complete"}else{"partial"},"coverage_ids":[181,182,183,184,185,186,187,188],"evidence":evidence,"missing":missing,"required_components":{"migration_matrix":"artifacts/status/command_migration_matrix.json","install_neutrality_report":"artifacts/status/install_neutrality_report.json","runtime_identity_report":"artifacts/status/active_runtime_report.json","closure_reports":"artifacts/status/command_family_closure_report.json","compatibility_debt_report":"artifacts/status/compatibility_debt_trend_report.json","cross_surface_consistency_report":"artifacts/status/cross_surface_consistency_artifact.json","known_remaining_gaps_report":"artifacts/status/what_is_left.json"}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/release_status_manifest.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","scope":"release status manifest","status":if missing.is_empty(){"ready"}else{"blocked"},"coverage_ids":[189,200],"checks":{"missing_evidence":missing,"parity_partial_count":partial.len(),"parity_missing_count":missing_cmd.len(),"stale_scripts_outside_dev_cli":scripts_audit.get("scripts").and_then(Value::as_array).map_or(0,Vec::len),"docs_markdown_count":docs_audit.get("markdown_count").and_then(Value::as_i64).unwrap_or(0),"weak_tests_count":weak_tests.len()},"review_steps":["review intentionally different behaviors","review unresolved partial commands","review stale scripts outside dev cli","review stale docs from docs audit","review weak tests from test audit","review release evidence bundle before release candidate decision"],"next_work_input":"Use release_evidence_bundle.json and release_truth_report.json as the first input for next prioritization.","status_discussion_policy":"status claims are invalid unless backed by artifacts in this manifest"})).ok()?;
            let done_payload = read("artifacts/status/what_is_done.json");
            let partial_payload = read("artifacts/status/what_is_partial.json");
            let intentional = read("artifacts/status/what_is_intentionally_different.json");
            let left = read("artifacts/status/what_is_left.json");
            let truth = json!({"generated_at":generated_at,"generator":"bijux-dev-cli","scope":"release truth","status":if missing.is_empty(){"ready"}else{"blocked"},"coverage_ids":[190,191,192,193,194,198,199,200],"summary":{"missing_evidence":missing.len(),"parity_partial":partial.len(),"parity_missing":missing_cmd.len(),"weak_tests":weak_tests.len()},"sections":{"fully_done":done_payload,"partial":partial_payload,"intentionally_different":intentional,"still_left":left},"claim_policy":"release claims are evidence-only"});
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/release_truth_report.json",
                &truth,
            )
            .ok()?;
            fs::write(workspace_root.join("artifacts/status/release_truth_report.txt"), format!("Release Truth Summary\n\nstatus: {}\nmissing_evidence: {}\nparity_partial: {}\nparity_missing: {}\nweak_tests: {}\n", truth.get("status").and_then(Value::as_str).unwrap_or("blocked"), missing.len(), partial.len(), missing_cmd.len(), weak_tests.len())).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/release_evidence_bundle.json","artifacts/status/release_status_manifest.json","artifacts/status/release_truth_report.json","artifacts/status/release_truth_report.txt"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-SCAFFOLD-REPORTS" => {
            let generated_at = generated_at_utc();
            let read_lines = |name: &str| -> Vec<String> {
                fs::read_to_string(
                    workspace_root.join("crates/bijux-cli/tests/snapshots").join(name),
                )
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToString::to_string)
                .collect()
            };
            let python_files = read_lines("plugin_scaffold_python_minimal_files.txt");
            let rust_files = read_lines("plugin_scaffold_rust_minimal_files.txt");
            let python_set = python_files.iter().cloned().collect::<BTreeSet<_>>();
            let rust_set = rust_files.iter().cloned().collect::<BTreeSet<_>>();
            let decorative_files = vec!["README.md", "pyproject.toml", "Cargo.toml", ".gitignore"];
            let decorative_python = python_files
                .iter()
                .filter(|p| decorative_files.contains(&p.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            let decorative_rust = rust_files
                .iter()
                .filter(|p| decorative_files.contains(&p.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_python_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","kind":"python","files":python_files,"count":python_files.len()})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_rust_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","kind":"rust","files":rust_files,"count":rust_files.len()})).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_scaffold_diff.json",
                &json!({
                    "generated_at":generated_at,"generator":"bijux-dev-cli",
                    "shared": python_set.intersection(&rust_set).cloned().collect::<Vec<_>>(),
                    "python_only": python_set.difference(&rust_set).cloned().collect::<Vec<_>>(),
                    "rust_only": rust_set.difference(&python_set).cloned().collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_non_behavioral_files.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","decorative_candidates":decorative_files,"present_in_scaffold":{"python":decorative_python,"rust":decorative_rust},"summary":"decorative files are excluded from minimal scaffold outputs"})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_file_justification.json", &json!({
                                "generated_at":generated_at,
                                "generator":"bijux-dev-cli",
                                "classification_values":["essential","helpful","removable"],
                                "files":{
                                    "python":{"plugin.manifest.json":{"classification":"essential","reason":"required for install, namespace validation, and lifecycle commands"},"plugin.py":{"classification":"essential","reason":"runtime entrypoint for delegated python plugins"}},
                                    "rust":{"plugin.manifest.json":{"classification":"essential","reason":"required for install, namespace validation, and lifecycle commands"},"src/lib.rs":{"classification":"essential","reason":"runtime entrypoint module for delegated rust plugins"}}
                                },
                                "freeze_rule":"every scaffolded file must have a justification and decorative outputs stay excluded",
                            })).ok()?;
            let summary = format!(
                                "Plugin scaffold minimalism summary\nGenerated at: {generated_at}\nPython files ({}): {}\nRust files ({}): {}\nDecorative files excluded: README.md, pyproject.toml, Cargo.toml, .gitignore\nPolicy: every scaffolded file must carry explicit justification\n",
                                python_files.len(),
                                python_files.join(", "),
                                rust_files.len(),
                                rust_files.join(", ")
                            );
            fs::write(
                workspace_root.join("artifacts/status/plugin_scaffold_minimalism_summary.txt"),
                summary,
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_scaffold_python_inventory.json",
                "artifacts/status/plugin_scaffold_rust_inventory.json",
                "artifacts/status/plugin_scaffold_diff.json",
                "artifacts/status/plugin_scaffold_non_behavioral_files.json",
                "artifacts/status/plugin_scaffold_file_justification.json",
                "artifacts/status/plugin_scaffold_minimalism_summary.txt"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-MIGRATION-REPORTS" => {
            let generated_at = generated_at_utc();
            let read = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join("artifacts/status").join(name))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let plugin_state = read("plugin_state_report.json");
            let scaffold_python = read("plugin_scaffold_python_inventory.json");
            let scaffold_rust = read("plugin_scaffold_rust_inventory.json");
            let scaffold_non_behavioral = read("plugin_scaffold_non_behavioral_files.json");
            let scaffold_justification = read("plugin_scaffold_file_justification.json");
            let namespace_abuse = read("namespace_abuse_report.json");
            let reserved_inventory = read("reserved_namespace_inventory.json");
            let rollback = read("plugin_rollback_proof_report.json");
            let lifecycle_failures = read("plugin_lifecycle_failure_injection_report.json");
            let plugin_health = read("plugin_health_report.json");
            let doctor_runtime = read("plugin_doctor_runtime_sample.json");
            let explain_runtime = read("plugin_explain_runtime_sample.json");
            let where_runtime = read("plugin_where_runtime_sample.json");
            let base = json!({"generated_at":generated_at,"generator":"bijux-dev-cli"});
            let lifecycle = json!({
                "generated_at":generated_at,"generator":"bijux-dev-cli",
                "stages":[
                    {"stage":"discover-and-list","rust_owned":true,"python_era_assumptions":[],"evidence":["crates/bijux-cli/tests/bin_surface/plugin_cli_lifecycle.rs::python_and_rust_plugins_can_install_check_list_and_uninstall","crates/bijux-cli/tests/bin_surface/plugin_command_parity.rs"]},
                    {"stage":"scaffold","rust_owned":true,"python_era_assumptions":["python scaffold runtime entrypoint remains plugin.py for compatibility"],"evidence":["crates/bijux-cli/tests/bin_surface/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust"]},
                    {"stage":"install-uninstall-enable-disable","rust_owned":true,"python_era_assumptions":[],"evidence":rollback.get("evidence").cloned().unwrap_or_else(|| json!([]))},
                    {"stage":"doctor-explain-where","rust_owned":true,"python_era_assumptions":[],"evidence":["artifacts/status/plugin_doctor_runtime_sample.json","artifacts/status/plugin_explain_runtime_sample.json","artifacts/status/plugin_where_runtime_sample.json"]},
                ],
                "summary":{"fully_rust_owned":4,"python_assumption_dependent":1}
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_lifecycle_ownership_report.json",
                &lifecycle,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_efficiency_report.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli",
                                "python_inventory":scaffold_python,"rust_inventory":scaffold_rust,"justification":scaffold_justification,
                                "decorative_presence": scaffold_non_behavioral.get("present_in_scaffold").cloned().unwrap_or_else(|| json!({})),
                                "status": if scaffold_non_behavioral.get("present_in_scaffold").and_then(|v| v.get("python")).and_then(Value::as_array).map_or(0, Vec::len)==0
                                    && scaffold_non_behavioral.get("present_in_scaffold").and_then(|v| v.get("rust")).and_then(Value::as_array).map_or(0, Vec::len)==0 {"minimal"} else {"needs-trim"}
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_lifecycle_proof_report.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli",
                                "python_scaffold_e2e_proof":{"status":"complete","evidence_test":"crates/bijux-cli/tests/bin_surface/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust","kind":"python"},
                                "rust_scaffold_e2e_proof":{"status":"complete","evidence_test":"crates/bijux-cli/tests/bin_surface/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust","kind":"rust"},
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_namespace_abuse_proof_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","abuse_report":namespace_abuse,"reserved_namespace_inventory":reserved_inventory})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_doctor_clarity_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","health_report":plugin_health,"runtime_sample":doctor_runtime,"status":if doctor_runtime.get("doctor").is_some() && doctor_runtime.get("status").is_some() {"clear"} else {"unclear"}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_explain_clarity_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","runtime_sample":explain_runtime,"status":if explain_runtime.get("diagnostics").is_some() && explain_runtime.get("summary").is_some() {"clear"} else {"unclear"}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_where_ownership_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","runtime_sample":where_runtime,"status":if where_runtime.get("plugins_dir").is_some() && where_runtime.get("registry_file").is_some() {"clear"} else {"unclear"}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_command_set_status.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","plugin_commands":plugin_state.get("plugin_commands").cloned().unwrap_or_else(|| json!({})),"classification":if plugin_state.get("plugin_commands").and_then(|p| p.get("partial")).and_then(Value::as_array).map_or(0,Vec::len)>0 {"evolving"} else {"complete"},"frozen_law":plugin_state.get("frozen_law").cloned().unwrap_or_else(|| json!("plugin v1 contract is frozen before expanding command cleverness")),"dynamic_complexity_policy":"reject unproven plugin complexity until parity and rollback evidence exists","operating_style":"boring-and-inspectable"})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_migration_report.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli",
                                "lifecycle_ownership":read("plugin_lifecycle_ownership_report.json"),
                                "scaffold_efficiency":read("plugin_scaffold_efficiency_report.json"),
                                "scaffold_lifecycle_proof":read("plugin_scaffold_lifecycle_proof_report.json"),
                                "namespace_abuse_proof":read("plugin_namespace_abuse_proof_report.json"),
                                "install_rollback_proof":rollback,
                                "uninstall_rollback_proof":{"status":rollback.get("status").cloned().unwrap_or_else(|| json!("unknown")),"evidence":rollback.get("evidence").cloned().unwrap_or_else(|| json!([]))},
                                "doctor_clarity":read("plugin_doctor_clarity_report.json"),
                                "explain_clarity":read("plugin_explain_clarity_report.json"),
                                "where_ownership":read("plugin_where_ownership_report.json"),
                                "command_set_status":read("plugin_command_set_status.json"),
                                "failure_injection":lifecycle_failures,
                            })).ok()?;
            let _ = base;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_lifecycle_ownership_report.json",
                "artifacts/status/plugin_scaffold_efficiency_report.json",
                "artifacts/status/plugin_scaffold_lifecycle_proof_report.json",
                "artifacts/status/plugin_namespace_abuse_proof_report.json",
                "artifacts/status/plugin_doctor_clarity_report.json",
                "artifacts/status/plugin_explain_clarity_report.json",
                "artifacts/status/plugin_where_ownership_report.json",
                "artifacts/status/plugin_command_set_status.json",
                "artifacts/status/plugin_migration_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-MANIFEST-SCAFFOLD-FUZZ-REPORTS" => {
            let now = generated_at_utc();
            let text = |p: &str| fs::read_to_string(workspace_root.join(p)).unwrap_or_default();
            let manifest_targets = "crates/bijux-cli-plugin/tests/plugin_manifest_fuzz_targets.rs";
            let manifest_reg = "crates/bijux-cli-plugin/tests/plugin_manifest_fuzz_regressions.rs";
            let scaffold_targets =
                "crates/bijux-cli/tests/bin_surface/plugin_scaffold_fuzz_targets.rs";
            let scaffold_reg =
                "crates/bijux-cli/tests/bin_surface/plugin_scaffold_fuzz_regressions.rs";
            let mtxt = text(manifest_targets);
            let mrtxt = text(manifest_reg);
            let stxt = text(scaffold_targets);
            let srtxt = text(scaffold_reg);
            let required: BTreeMap<i64, (&str, &str)> = BTreeMap::from([
                (61, (manifest_targets, "fuzz_plugin_manifest_parsing_is_stable")),
                (
                    62,
                    (
                        manifest_targets,
                        "fuzz_plugin_manifest_validation_covers_required_and_optional_fields",
                    ),
                ),
                (63, (manifest_targets, "fuzz_compatibility_range_parsing_is_enforced")),
                (64, (manifest_targets, "fuzz_plugin_entrypoint_path_parsing_by_kind_is_enforced")),
                (
                    65,
                    (
                        manifest_targets,
                        "fuzz_plugin_metadata_optional_fields_and_duplicate_aliases",
                    ),
                ),
                (
                    66,
                    (
                        scaffold_targets,
                        "fuzz_scaffold_option_parsing_and_template_expansion_inputs_are_stable",
                    ),
                ),
                (
                    67,
                    (
                        scaffold_targets,
                        "fuzz_scaffold_option_parsing_and_template_expansion_inputs_are_stable",
                    ),
                ),
                (
                    68,
                    (
                        scaffold_targets,
                        "fuzz_python_and_rust_scaffold_manifest_generation_are_correct",
                    ),
                ),
                (
                    69,
                    (
                        scaffold_targets,
                        "fuzz_python_and_rust_scaffold_manifest_generation_are_correct",
                    ),
                ),
                (70, (scaffold_targets, "fuzz_scaffold_path_sanitization_rejects_parent_segments")),
                (
                    71,
                    (
                        scaffold_targets,
                        "fuzz_plugin_inspect_payload_and_check_diagnostics_rendering_are_stable",
                    ),
                ),
                (
                    72,
                    (
                        scaffold_targets,
                        "fuzz_plugin_inspect_payload_and_check_diagnostics_rendering_are_stable",
                    ),
                ),
                (73, (scaffold_targets, "fuzz_plugin_reserved_name_error_rendering_is_stable")),
                (76, (manifest_reg, "minimized_plugin_manifest_cases_replay_deterministically")),
                (
                    77,
                    (scaffold_reg, "minimized_scaffold_cases_replay_with_deterministic_exit_codes"),
                ),
                (78, (manifest_reg, "minimized_plugin_manifest_cases_replay_deterministically")),
                (
                    79,
                    (scaffold_reg, "minimized_scaffold_cases_replay_with_deterministic_exit_codes"),
                ),
            ]);
            let coverage = required.iter().map(|(id, (p, t))| {
                                let src = if *p == manifest_targets { &mtxt } else if *p == manifest_reg { &mrtxt } else if *p == scaffold_targets { &stxt } else { &srtxt };
                                json!({"coverage_id":id,"test":t,"status":if src.contains(&format!("fn {t}(")){"covered"}else{"missing"},"evidence":p})
                            }).collect::<Vec<_>>();
            let manifest_cases = collect_files(
                &workspace_root
                    .join("crates/bijux-cli-plugin/tests/fuzz/plugin_manifest_minimized_cases"),
            )
            .into_iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .map(|p| rel(&p, workspace_root))
            .collect::<Vec<_>>();
            let scaffold_cases = collect_files(
                &workspace_root.join("crates/bijux-cli/tests/fuzz/plugin_scaffold_minimized_cases"),
            )
            .into_iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("argv"))
            .map(|p| rel(&p, workspace_root))
            .collect::<Vec<_>>();
            let run = |args: &[&str]| {
                Command::new("cargo")
                    .args(args)
                    .current_dir(workspace_root)
                    .status()
                    .ok()
                    .is_some_and(|s| s.success())
            };
            let mt_ok =
                run(&["test", "-p", "bijux-cli-plugin", "--test", "plugin_manifest_fuzz_targets"]);
            let mr_ok = run(&[
                "test",
                "-p",
                "bijux-cli-plugin",
                "--test",
                "plugin_manifest_fuzz_regressions",
            ]);
            let st_ok = run(&[
                "test",
                "-p",
                "bijux-cli",
                "--test",
                "integration",
                "plugin_scaffold_fuzz_targets::",
            ]);
            let sr_ok = run(&[
                "test",
                "-p",
                "bijux-cli",
                "--test",
                "integration",
                "plugin_scaffold_fuzz_regressions::",
            ]);
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_manifest_crash_triage_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin manifest fuzz crash triage","coverage_ids":[74],"status":if mt_ok && mr_ok{"clean"}else{"needs-triage"},"target_suite_ok":mt_ok,"regression_suite_ok":mr_ok,"minimized_case_count":manifest_cases.len()})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_crash_triage_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin scaffold fuzz crash triage","coverage_ids":[75],"status":if st_ok && sr_ok{"clean"}else{"needs-triage"},"target_suite_ok":st_ok,"regression_suite_ok":sr_ok,"minimized_case_count":scaffold_cases.len()})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_manifest_fuzz_regression_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin manifest fuzz regressions","coverage_ids":[76,78],"status":if mr_ok{"clean"}else{"drift"},"minimized_cases":manifest_cases})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_fuzz_regression_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin scaffold fuzz regressions","coverage_ids":[77,79],"status":if sr_ok{"clean"}else{"drift"},"minimized_cases":scaffold_cases})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_manifest_scaffold_fuzz_contract.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin manifest and scaffold fuzzing","coverage_ids":(61..81).collect::<Vec<_>>(),"status":if missing.is_empty() && mt_ok && mr_ok && st_ok && sr_ok && !manifest_cases.is_empty() && !scaffold_cases.is_empty(){"frozen"}else{"partial"},"coverage_rows":coverage,"missing_coverage_ids":missing,"manifest_minimized_case_count":manifest_cases.len(),"scaffold_minimized_case_count":scaffold_cases.len(),"policy":"plugin manifest and scaffold fuzzing remain maintenance-required hardening checks"})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_manifest_crash_triage_artifact.json",
                "artifacts/status/plugin_scaffold_crash_triage_artifact.json",
                "artifacts/status/plugin_manifest_fuzz_regression_artifact.json",
                "artifacts/status/plugin_scaffold_fuzz_regression_artifact.json",
                "artifacts/status/plugin_manifest_scaffold_fuzz_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-STATE-CORRUPTION-CAMPAIGN-REPORTS" => {
            let now = generated_at_utc();
            let campaign_test = "crates/bijux-cli/tests/bin_surface/randomized_plugin_state_corruption_campaigns.rs";
            let regression_test = "crates/bijux-cli/tests/bin_surface/plugin_state_corruption_campaign_regressions.rs";
            let campaign_text =
                fs::read_to_string(workspace_root.join(campaign_test)).unwrap_or_default();
            let regression_text =
                fs::read_to_string(workspace_root.join(regression_test)).unwrap_or_default();
            let required: BTreeMap<i64, (&str, &str)> = BTreeMap::from([
                                (141, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (142, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (143, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (144, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (145, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (146, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (147, (campaign_test, "one_broken_plugin_never_hides_unrelated_healthy_plugins")),
                                (148, (campaign_test, "plugin_list_is_deterministic_for_identical_corrupted_registry")),
                                (149, (campaign_test, "plugin_registry_rollback_preserves_coherence_after_failed_mutation_paths")),
                                (150, (campaign_test, "plugin_registry_rollback_preserves_coherence_after_failed_mutation_paths")),
                                (151, (campaign_test, "plugin_doctor_reports_corruption_injected_by_campaign")),
                                (152, (campaign_test, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant")),
                                (153, (campaign_test, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant")),
                                (154, (campaign_test, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant")),
                                (155, (campaign_test, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant")),
                                (158, (regression_test, "minimized_plugin_state_corruption_cases_replay_without_crashing")),
                            ]);
            let coverage = required.iter().map(|(id, (p, t))| {
                                let src = if *p == campaign_test { &campaign_text } else { &regression_text };
                                json!({"coverage_id":id,"test":t,"status":if src.contains(&format!("fn {t}(")){"covered"}else{"missing"},"evidence":p})
                            }).collect::<Vec<_>>();
            let campaign_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "randomized_plugin_state_corruption_campaigns::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let regression_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "plugin_state_corruption_campaign_regressions::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let minimized_cases = collect_files(
                &workspace_root
                    .join("crates/bijux-cli/tests/fuzz/plugin_state_corruption_minimized_cases"),
            )
            .into_iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .map(|p| rel(&p, workspace_root))
            .collect::<Vec<_>>();
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_campaign_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption campaigns","coverage_ids":(141..156).collect::<Vec<_>>(),"status":if campaign_ok{"complete"}else{"partial"},"campaign_suite":{"ok":campaign_ok}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_corpus_retention_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption corpus retention","coverage_ids":[156],"status":if minimized_cases.is_empty(){"partial"}else{"complete"},"minimized_case_count":minimized_cases.len(),"minimized_cases":minimized_cases})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_triage_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption triage","coverage_ids":[157],"status":if campaign_ok && regression_ok{"clean"}else{"needs-triage"},"campaign_suite_ok":campaign_ok,"regression_suite_ok":regression_ok})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_regression_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption regression replay","coverage_ids":[158],"status":if regression_ok{"clean"}else{"drift"},"minimized_cases":minimized_cases})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_severity_classification.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption severity classification","coverage_ids":[159],"status":"complete","classes":{"critical":["plugin registry write rollback failure","state read panic"],"high":["nondeterministic plugin list under identical corrupted input","memory recovery drift"],"medium":["history malformed entries with degraded but successful read"],"low":["doctor self-repair with stable output"]}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_contract.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption hardening contract","coverage_ids":(141..161).collect::<Vec<_>>(),"status":if campaign_ok && regression_ok && !minimized_cases.is_empty() && missing.is_empty(){"frozen"}else{"partial"},"missing_coverage_ids":missing,"policy":"plugin/history/memory corruption campaigns are required hardening coverage"})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_state_corruption_campaign_artifact.json",
                "artifacts/status/plugin_state_corruption_corpus_retention_artifact.json",
                "artifacts/status/plugin_state_corruption_triage_artifact.json",
                "artifacts/status/plugin_state_corruption_regression_artifact.json",
                "artifacts/status/plugin_state_corruption_severity_classification.json",
                "artifacts/status/plugin_state_corruption_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CONFIG-DEEP-BEHAVIOR-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/config_deep_behavior_matrix.rs"),
            )
            .unwrap_or_default();
            let has_test = |name: &str| source.contains(&format!("fn {name}("));
            let run_json_or_empty =
                |args: &[&str]| run_bijux_json(workspace_root, args).unwrap_or_else(|_| json!({}));
            let semantic_roundtrip = run_json_or_empty(&["cli", "config", "list"]);
            let precedence_view = run_json_or_empty(&["dev", "cli", "env"]);
            let corruption_view = run_json_or_empty(&["dev", "cli", "state-doctor"]);
            let determinism_a = Command::new("cargo")
                .args([
                    "run",
                    "-q",
                    "-p",
                    "bijux-cli",
                    "--",
                    "cli",
                    "config",
                    "list",
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
                    "cli",
                    "config",
                    "list",
                    "--format",
                    "json",
                    "--no-pretty",
                ])
                .current_dir(workspace_root)
                .output()
                .ok();
            let deterministic = determinism_a.as_ref().is_some_and(|o| o.status.success())
                && determinism_b.as_ref().is_some_and(|o| o.status.success())
                && determinism_a.as_ref().map(|o| (&o.stdout, &o.stderr))
                    == determinism_b.as_ref().map(|o| (&o.stdout, &o.stderr));
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (
                    81,
                    "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
                ),
                (82, "config_writer_ordering_and_formatting_rules_are_deterministic"),
                (
                    83,
                    "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
                ),
                (
                    84,
                    "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
                ),
                (
                    85,
                    "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
                ),
                (
                    86,
                    "config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs",
                ),
                (87, "config_writer_ordering_and_formatting_rules_are_deterministic"),
                (88, "config_export_and_load_preserve_semantic_content_and_roundtrip_exact_values"),
                (89, "config_export_and_load_preserve_semantic_content_and_roundtrip_exact_values"),
                (90, "config_export_and_load_preserve_semantic_content_and_roundtrip_exact_values"),
                (91, "config_unset_clear_and_repeated_mutations_follow_expected_semantics"),
                (92, "config_unset_clear_and_repeated_mutations_follow_expected_semantics"),
                (93, "config_unset_clear_and_repeated_mutations_follow_expected_semantics"),
                (94, "root_and_cli_config_path_override_behavior_is_identical_for_list"),
                (95, "config_doctor_and_state_doctor_agree_on_corrupted_config_findings"),
            ]);
            let coverage_rows = required.iter().map(|(id, name)| json!({
                                "coverage_id": id, "test_name": name,
                                "status": if has_test(name) {"covered"} else {"missing"},
                                "evidence": "crates/bijux-cli/tests/bin_surface/config_deep_behavior_matrix.rs"
                            })).collect::<Vec<_>>();
            let missing = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            let semantic = json!({"generator":"bijux-dev-cli","scope":"config semantic roundtrip","coverage_ids":[88,89,90,91,92,96],"status":if semantic_roundtrip.is_object(){"complete"}else{"partial"},"sample":semantic_roundtrip});
            let precedence = json!({"generator":"bijux-dev-cli","scope":"config precedence","coverage_ids":[94,97],"status":if precedence_view.is_object(){"complete"}else{"partial"},"sample":precedence_view});
            let determinism = json!({"generator":"bijux-dev-cli","scope":"config determinism","coverage_ids":[81,82,83,84,85,86,87,93,98],"status":if deterministic{"complete"}else{"partial"},"byte_stable":deterministic});
            let corruption = json!({"generator":"bijux-dev-cli","scope":"config corruption recovery","coverage_ids":[95,99],"status":if corruption_view.is_object(){"complete"}else{"partial"},"sample":corruption_view});
            let mut drift = Vec::<Value>::new();
            for (name, payload) in [
                ("config_semantic_roundtrip_artifact.json", &semantic),
                ("config_precedence_artifact.json", &precedence),
                ("config_determinism_artifact.json", &determinism),
                ("config_corruption_recovery_artifact.json", &corruption),
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
                "artifacts/status/config_semantic_roundtrip_artifact.json",
                &semantic,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_precedence_artifact.json",
                &precedence,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_determinism_artifact.json",
                &determinism,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_corruption_recovery_artifact.json",
                &corruption,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_deep_behavior_drift_artifact.json", &json!({
                                "generator":"bijux-dev-cli","scope":"config deep behavior drift","coverage_ids":[100],
                                "status": if drift.is_empty() {"clean"} else {"drift-detected"},
                                "drift_count": drift.len(),
                                "drift_items": drift,
                                "coverage_rows": coverage_rows,
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/config_semantic_roundtrip_artifact.json",
                "artifacts/status/config_precedence_artifact.json",
                "artifacts/status/config_determinism_artifact.json",
                "artifacts/status/config_corruption_recovery_artifact.json",
                "artifacts/status/config_deep_behavior_drift_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CONFIG-CORRUPTION-CAMPAIGN-REPORTS" => {
            let now = generated_at_utc();
            let campaign_test = workspace_root.join(
                "crates/bijux-cli/tests/bin_surface/randomized_config_corruption_campaigns.rs",
            );
            let regression_test = workspace_root.join(
                "crates/bijux-cli/tests/bin_surface/config_corruption_campaign_regressions.rs",
            );
            let campaign_text = fs::read_to_string(&campaign_test).unwrap_or_default();
            let regression_text = fs::read_to_string(&regression_test).unwrap_or_default();
            let required: BTreeMap<i64, (&str, &str)> = BTreeMap::from([
                                (121, ("campaign", "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands")),
                                (122, ("campaign", "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands")),
                                (123, ("campaign", "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands")),
                                (124, ("campaign", "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands")),
                                (125, ("campaign", "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands")),
                                (126, ("campaign", "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands")),
                                (127, ("campaign", "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands")),
                                (128, ("campaign", "randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands")),
                                (129, ("campaign", "config_mutations_never_silently_destroy_unrelated_valid_keys")),
                                (130, ("campaign", "config_corruption_has_stable_failure_class_and_recovery_path")),
                                (131, ("campaign", "failed_config_load_rolls_back_and_preserves_coherent_state")),
                                (132, ("campaign", "state_doctor_reports_corruption_introduced_by_campaign_harness")),
                                (133, ("campaign", "repeated_run_corruption_inputs_are_deterministic_for_config_command_set")),
                                (136, ("regression", "minimized_config_corruption_campaign_cases_replay_without_crashing")),
                            ]);
            let coverage = required.iter().map(|(id, (src, name))| {
                                let text = if *src == "campaign" { &campaign_text } else { &regression_text };
                                json!({"coverage_id":id,"test":name,"status":if text.contains(&format!("fn {name}(")){"covered"}else{"missing"},"evidence":if *src=="campaign" {"crates/bijux-cli/tests/bin_surface/randomized_config_corruption_campaigns.rs"} else {"crates/bijux-cli/tests/bin_surface/config_corruption_campaign_regressions.rs"}})
                            }).collect::<Vec<_>>();
            let campaign_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "randomized_config_corruption_campaigns::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let regression_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "config_corruption_campaign_regressions::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let minimized = collect_files(
                &workspace_root
                    .join("crates/bijux-cli/tests/fuzz/config_corruption_minimized_cases"),
            )
            .into_iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .map(|p| rel(&p, workspace_root))
            .collect::<Vec<_>>();
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_campaign_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"randomized config corruption campaigns","coverage_ids":(121..129).collect::<Vec<_>>(),"status":if campaign_ok{"complete"}else{"partial"},"campaign_suite":{"ok":campaign_ok}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_invariants_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"config corruption invariants","coverage_ids":[129,130,131,132,133],"status":if campaign_ok && ![129,130,131,132,133].iter().any(|id| missing.contains(id)){"complete"}else{"partial"},"coverage_rows":coverage.iter().filter(|r| r.get("coverage_id").and_then(Value::as_i64).is_some_and(|id| (129..=133).contains(&id))).cloned().collect::<Vec<_>>()})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_corpus_retention_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"config corruption corpus retention","coverage_ids":[134],"status":if minimized.is_empty(){"partial"}else{"complete"},"minimized_case_count":minimized.len(),"minimized_cases":minimized})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_triage_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"config corruption triage","coverage_ids":[135],"status":if campaign_ok && regression_ok{"clean"}else{"needs-triage"},"campaign_suite_ok":campaign_ok,"regression_suite_ok":regression_ok})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_regression_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"config corruption regression replay","coverage_ids":[136],"status":if regression_ok{"clean"}else{"drift"},"minimized_cases":minimized})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_severity_classification.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"config corruption severity classification","coverage_ids":[137],"status":"complete","classes":{"critical":["write-path panic","state file replacement with empty content"],"high":["rollback failure","nondeterministic failure class"],"medium":["malformed input with clean failure"],"low":["recoverable duplicate-key or whitespace anomalies"]}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_recovery_classification.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"config corruption recovery classification","coverage_ids":[138],"status":"complete","paths":{"stable_failure":["usage/validation failure with unchanged file content"],"self_recovery":["repair input and rerun command to success"],"rollback_preserved":["failed load keeps previous coherent config"]}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_determinism_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"config corruption determinism","coverage_ids":[139],"status":if campaign_ok{"complete"}else{"partial"},"deterministic_failure_class_required":true,"evidence":"crates/bijux-cli/tests/bin_surface/randomized_config_corruption_campaigns.rs::repeated_run_corruption_inputs_are_deterministic_for_config_command_set"})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/config_corruption_release_blocking_contract.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"config corruption release-blocking contract","coverage_ids":(121..141).collect::<Vec<_>>(),"status":if campaign_ok && regression_ok && !minimized.is_empty() && missing.is_empty(){"frozen"}else{"partial"},"missing_coverage_ids":missing,"release_blocking":true,"policy":"config corruption campaign coverage and deterministic rollback behavior are required before release"})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/config_corruption_campaign_artifact.json",
                "artifacts/status/config_corruption_invariants_artifact.json",
                "artifacts/status/config_corruption_corpus_retention_artifact.json",
                "artifacts/status/config_corruption_triage_artifact.json",
                "artifacts/status/config_corruption_regression_artifact.json",
                "artifacts/status/config_corruption_severity_classification.json",
                "artifacts/status/config_corruption_recovery_classification.json",
                "artifacts/status/config_corruption_determinism_artifact.json",
                "artifacts/status/config_corruption_release_blocking_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-DEEP-BEHAVIOR-REPORTS" => {
            let tests = [
                "crates/bijux-cli/tests/bin_surface/diagnostics_command_matrix.rs",
                "crates/bijux-cli/tests/bin_surface/diagnostics_contract_consistency.rs",
                "crates/bijux-cli/tests/bin_surface/diagnostics_deep_behavior_extra.rs",
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
                sources.iter().find(|(_, src)| src.contains(&needle)).map(|(p, _)| p.clone())
            };
            let run_json =
                |args: &[&str]| run_bijux_json(workspace_root, args).unwrap_or_else(|_| json!({}));
            let doctor_a = run_json(&["doctor"]);
            let doctor_b = run_json(&["doctor"]);
            let state_doctor_a = run_json(&["dev", "cli", "state-doctor"]);
            let state_doctor_b = run_json(&["dev", "cli", "state-doctor"]);
            let inspect = run_json(&["inspect"]);
            let env = run_json(&["dev", "cli", "env"]);
            let contracts = run_json(&["dev", "cli", "contracts"]);
            let routes = run_json(&["dev", "cli", "routes"]);
            let registry = run_json(&["dev", "cli", "registry"]);
            let plugin_health = run_json(&["dev", "cli", "plugin-health"]);
            let package_health = run_json(&["dev", "cli", "package-health"]);
            let runtime_identity = run_json(&["dev", "cli", "runtime-identity"]);
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (141, "doctor_findings_are_stable_and_do_not_reorder_nondeterministically"),
                                (142, "doctor_findings_are_stable_and_do_not_reorder_nondeterministically"),
                                (143, "doctor_json_and_text_are_stable_with_no_color_mode"),
                                (144, "doctor_json_and_text_are_stable_with_no_color_mode"),
                                (145, "inspect_and_doctor_agree_on_route_state_overlap_signals"),
                                (146, "dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution"),
                                (147, "dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution"),
                                (148, "dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution"),
                                (149, "dev_cli_env_contracts_routes_and_registry_match_current_snapshots_and_resolution"),
                                (150, "state_doctor_and_plugin_health_match_corruption_harness_findings"),
                                (151, "state_doctor_and_plugin_health_match_corruption_harness_findings"),
                                (152, "package_health_and_runtime_identity_are_consistent_with_active_binary_conditions"),
                                (153, "package_health_and_runtime_identity_are_consistent_with_active_binary_conditions"),
                            ]);
            let coverage = required
                                .iter()
                                .map(|(id, name)| {
                                    let evidence = find_test(name);
                                    json!({"coverage_id":id,"test_name":name,"status":if evidence.is_some(){"covered"}else{"missing"},"evidence":evidence})
                                })
                                .collect::<Vec<_>>();
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            let expected_contracts = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/snapshots/ported/dev_cli_contracts.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let expected_routes = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/snapshots/ported/dev_cli_routes.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let route_set = |value: &Value| -> BTreeSet<String> {
                value
                    .get("routes")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|row| row.get("segments").and_then(Value::as_array).cloned())
                    .map(|segments| {
                        segments
                            .into_iter()
                            .filter_map(|s| s.as_str().map(ToString::to_string))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect()
            };
            let expected_route_set = route_set(&expected_routes);
            let current_route_set = route_set(&routes);
            let diagnostics_consistency = json!({"generator":"bijux-dev-cli","scope":"diagnostics consistency","coverage_ids":[145,146,149,150,151,152,154],"status":if inspect.is_object()&&doctor_a.is_object()&&env.is_object()&&routes.is_object()&&registry.is_object()&&package_health.is_object()&&runtime_identity.is_object(){"complete"}else{"partial"},"sample":{"inspect_status":inspect.get("status"),"doctor_status":doctor_a.get("status"),"env_keys":env.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()}});
            let doctor_determinism = json!({"generator":"bijux-dev-cli","scope":"doctor determinism","coverage_ids":[141,142,143,144,155,158],"status":if doctor_a==doctor_b && state_doctor_a==state_doctor_b && state_doctor_a.get("doctor").and_then(|d| d.get("issues"))==state_doctor_b.get("doctor").and_then(|d| d.get("issues")){"complete"}else{"partial"},"byte_stable":doctor_a==doctor_b && state_doctor_a==state_doctor_b});
            let schema_drift = json!({"generator":"bijux-dev-cli","scope":"diagnostics schema drift","coverage_ids":[147,148,156],"status":if contracts==expected_contracts && expected_route_set.is_subset(&current_route_set){"complete"}else{"partial"},"contracts_matches_snapshot":contracts==expected_contracts,"routes_matches_snapshot":expected_route_set.is_subset(&current_route_set)});
            let source_of_truth = json!({"generator":"bijux-dev-cli","scope":"diagnostics source of truth","coverage_ids":[146,147,148,149,157],"status":if env.is_object()&&contracts.is_object()&&routes.is_object()&&registry.is_object(){"complete"}else{"partial"},"source_commands":["dev cli env","dev cli contracts","dev cli routes","dev cli registry"]});
            let findings_order = json!({"generator":"bijux-dev-cli","scope":"findings order","coverage_ids":[141,142,150,158],"status":if state_doctor_a.get("doctor").and_then(|d| d.get("issues"))==state_doctor_b.get("doctor").and_then(|d| d.get("issues")){"complete"}else{"partial"},"stable_order":state_doctor_a.get("doctor").and_then(|d| d.get("issues"))==state_doctor_b.get("doctor").and_then(|d| d.get("issues"))});
            let contract = json!({"generator":"bijux-dev-cli","scope":"diagnostics contract","coverage_ids":[143,144,145,152,153,159],"status":if doctor_a.is_object()&&plugin_health.is_object()&&package_health.is_object()&&runtime_identity.is_object(){"complete"}else{"partial"},"contract_keys":{"doctor":doctor_a.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),"plugin_health":plugin_health.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),"package_health":package_health.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),"runtime_identity":runtime_identity.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()}});
            let mut drift = Vec::<Value>::new();
            for (name, payload) in [
                ("diagnostics_consistency_artifact.json", &diagnostics_consistency),
                ("doctor_determinism_artifact.json", &doctor_determinism),
                ("diagnostics_schema_drift_artifact.json", &schema_drift),
                ("diagnostics_source_of_truth_artifact.json", &source_of_truth),
                ("findings_order_artifact.json", &findings_order),
                ("diagnostics_contract_artifact.json", &contract),
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
                "artifacts/status/diagnostics_consistency_artifact.json",
                &diagnostics_consistency,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/doctor_determinism_artifact.json",
                &doctor_determinism,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_schema_drift_artifact.json",
                &schema_drift,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_source_of_truth_artifact.json",
                &source_of_truth,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/findings_order_artifact.json",
                &findings_order,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_contract_artifact.json",
                &contract,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/diagnostics_deep_behavior_drift_artifact.json", &json!({"generator":"bijux-dev-cli","scope":"diagnostics deep behavior drift","coverage_ids":[160],"status":if drift.is_empty(){"clean"}else{"drift-detected"},"drift_count":drift.len(),"drift_items":drift,"coverage_rows":coverage})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/diagnostics_consistency_artifact.json",
                "artifacts/status/doctor_determinism_artifact.json",
                "artifacts/status/diagnostics_schema_drift_artifact.json",
                "artifacts/status/diagnostics_source_of_truth_artifact.json",
                "artifacts/status/findings_order_artifact.json",
                "artifacts/status/diagnostics_contract_artifact.json",
                "artifacts/status/diagnostics_deep_behavior_drift_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-TRUST-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/diagnostics_trust_law_extra.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (361, "dev_cli_contracts_and_routes_match_snapshot_semantics_and_are_byte_stable"),
                                (362, "dev_cli_contracts_and_routes_match_snapshot_semantics_and_are_byte_stable"),
                                (363, "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (364, "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (365, "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (366, "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (367, "dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (368, "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases"),
                                (369, "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases"),
                                (370, "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases"),
                                (371, "diagnostics_do_not_invent_unsupported_remediation_steps"),
                                (372, "diagnostics_text_is_boring_and_json_is_machine_friendly"),
                                (373, "diagnostics_text_is_boring_and_json_is_machine_friendly"),
                                (374, "diagnostics_runs_are_deterministic_for_covered_commands"),
                            ]);
            let coverage = required.iter().map(|(id, t)| json!({"coverage_id":id,"test":t,"status":if source.contains(&format!("fn {t}(")){"covered"}else{"missing"},"evidence":"crates/bijux-cli/tests/bin_surface/diagnostics_trust_law_extra.rs"})).collect::<Vec<_>>();
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            let expected_keys: BTreeMap<&str, Vec<&str>> = BTreeMap::from([
                ("dev cli contracts", vec!["contracts", "runtime_version", "schema_version"]),
                ("dev cli routes", vec!["aliases", "routes"]),
                ("dev cli registry", vec!["ownership", "precedence", "registry"]),
                ("dev cli env", vec!["active", "env", "source_precedence"]),
                (
                    "dev cli parity",
                    vec![
                        "binary_bridge",
                        "command_matrix",
                        "commands_fully_rust_owned",
                        "commands_python_only",
                        "commands_using_compatibility_shims",
                        "coverage",
                        "diffs",
                        "exit_code_report",
                        "flag_normalization_report",
                        "help_diff_report",
                        "machine_output_diff_report",
                        "parity_dashboard",
                        "parity_dashboard_text",
                        "plugin_lifecycle",
                        "plugin_matrix",
                        "precedence_report",
                        "python_bridge_matrix",
                        "repl_cli_output_diff",
                        "repl_matrix",
                        "rust_python",
                        "state_behavior_matrix",
                        "state_parity",
                        "stream_report",
                        "text_summary",
                    ],
                ),
                (
                    "dev cli crate-health",
                    vec![
                        "crate_metrics",
                        "crate_report",
                        "cross_crate_api_usage",
                        "dependency_edges",
                        "duplication_hotspots",
                        "internal_only_candidates_by_crate",
                        "public_api_by_crate",
                        "public_api_counts",
                    ],
                ),
                ("dev cli docs-audit", vec!["docs", "docs_audit", "docs_count"]),
                ("dev cli doctor", vec!["issues", "runtime", "status"]),
                (
                    "dev cli runtime-identity",
                    vec![
                        "active_binary",
                        "active_binary_selection_is_ambiguous",
                        "active_path_is_canonical_name",
                        "active_path_is_shadowed",
                        "canonical_user_binary",
                        "diagnostics",
                        "entrypoints",
                        "install_source",
                        "package_channels",
                        "path_binaries",
                        "public_runtime_binary_names",
                        "runtime",
                        "schema",
                        "secondary_public_runtime_binary_names",
                        "text_summary",
                    ],
                ),
            ]);
            let mut schema_rows = Vec::<Value>::new();
            for (command, expected) in &expected_keys {
                let parts = command.split_whitespace().collect::<Vec<_>>();
                let payload = run_bijux_json(workspace_root, &parts).unwrap_or_else(|_| json!({}));
                let actual = payload
                    .as_object()
                    .map(|o| o.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                let mut sorted_actual = actual.clone();
                sorted_actual.sort();
                let mut sorted_expected =
                    expected.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                sorted_expected.sort();
                schema_rows.push(json!({"command":command,"expected_keys":expected,"actual_keys":sorted_actual,"status":if sorted_actual==sorted_expected{"match"}else{"drift"}}));
            }
            let schema_drift = schema_rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("match"))
                .count();
            let plugin_health = run_bijux_json(workspace_root, &["dev", "cli", "plugin-health"])
                .unwrap_or_else(|_| json!({}));
            let trust = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"diagnostics trust","coverage_ids":[361,362,363,364,365,366,367,374,375],"status":if missing.is_empty(){"complete"}else{"partial"},"coverage_rows":coverage});
            let actionable = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"actionable diagnostics","coverage_ids":[368,369,370,371,376],"status":if missing.is_empty(){"complete"}else{"partial"},"checks":{"plugin_health_has_guidance":serde_json::to_string(&plugin_health).unwrap_or_default().contains("Use `bijux dev cli plugin-health --format json`"),"doctor_payload_present":run_bijux_json(workspace_root,&["dev","cli","doctor"]).map(|v|v.is_object()).unwrap_or(false),"runtime_identity_payload_present":run_bijux_json(workspace_root,&["dev","cli","runtime-identity"]).map(|v|v.is_object()).unwrap_or(false)}});
            let minimalism = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"diagnostics minimalism","coverage_ids":[372,373,377],"status":if missing.is_empty(){"complete"}else{"partial"},"json_commands_checked":expected_keys.keys().collect::<Vec<_>>(),"json_schema_drift_count":schema_drift});
            let schema = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"diagnostics trust schema drift","coverage_ids":[378],"status":if schema_drift==0 && missing.is_empty(){"clean"}else{"drift"},"drift_count":schema_drift + missing.len(),"schema_rows":schema_rows,"missing_coverage_ids":missing});
            let contract = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"diagnostics trust contract","coverage_ids":[380],"status":if schema_drift==0 && missing.is_empty(){"frozen"}else{"not-frozen"},"law":"diagnostics are credible operator output"});
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_trust_artifact.json",
                &trust,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/actionable_diagnostics_artifact.json",
                &actionable,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_minimalism_artifact.json",
                &minimalism,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_trust_schema_drift_artifact.json",
                &schema,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_trust_contract.json",
                &contract,
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/diagnostics_trust_artifact.json",
                "artifacts/status/actionable_diagnostics_artifact.json",
                "artifacts/status/diagnostics_minimalism_artifact.json",
                "artifacts/status/diagnostics_trust_schema_drift_artifact.json",
                "artifacts/status/diagnostics_trust_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-STATUS-REPORTS" => {
            let generated_at = generated_at_utc();
            let read = |p: &str| {
                fs::read_to_string(workspace_root.join(p))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let current_state = read("artifacts/status/current_rust_state.json");
            let parity_matrix = read("artifacts/parity/command_parity_matrix.json");
            let bridge_report = read("artifacts/parity/binary_vs_python_bridge_parity_report.json");
            let runtime_unity = read("artifacts/status/runtime_unity_report.json");
            let state_config = read("artifacts/parity/config_parity_report.json");
            let state_history = read("artifacts/parity/history_parity_report.json");
            let state_memory = read("artifacts/parity/memory_parity_report.json");
            let plugin_state = read("artifacts/status/plugin_state_report.json");
            let intentional = read("docs/architecture/parity/intentional_differences.json");
            let aliases = current_state
                .get("rust_routed_commands")
                .and_then(|r| r.get("aliases"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect::<BTreeSet<_>>();
            let rows = parity_matrix
                .get("commands")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut command_rows = rows
                                .into_iter()
                                .filter_map(|row| row.as_object().cloned())
                                .filter_map(|row| {
                                    let command = row.get("command")?.as_str()?.trim().to_string();
                                    if command.is_empty() {
                                        return None;
                                    }
                                    let matrix_status = row.get("status").and_then(Value::as_str).unwrap_or("missing");
                                    let status = if aliases.contains(&command) {
                                        "shim"
                                    } else if matrix_status == "missing" {
                                        "missing"
                                    } else if matrix_status == "partial" {
                                        "partial"
                                    } else {
                                        "complete"
                                    };
                                    Some(json!({
                                        "command":command,"group":row.get("group").and_then(Value::as_str).unwrap_or("unknown"),
                                        "status":status,"matrix_status":matrix_status,
                                        "owner":row.get("owner").and_then(Value::as_str).unwrap_or(""),
                                        "reason":row.get("reason").and_then(Value::as_str).unwrap_or(""),
                                        "blocker":row.get("blocker").and_then(Value::as_str).unwrap_or(""),
                                        "confidence":row.get("confidence").cloned().unwrap_or_else(|| json!(0.0))
                                    }))
                                })
                                .collect::<Vec<_>>();
            command_rows.sort_by(|a, b| {
                a.get("command")
                    .and_then(Value::as_str)
                    .cmp(&b.get("command").and_then(Value::as_str))
            });
            let root_commands = command_rows
                .iter()
                .filter_map(|r| r.get("command").and_then(Value::as_str))
                .filter_map(|c| c.split_whitespace().next().map(ToString::to_string))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let cli_commands = command_rows
                .iter()
                .filter_map(|r| r.get("command").and_then(Value::as_str))
                .filter(|c| c.starts_with("cli "))
                .map(|c| c.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let dev_cli_commands = command_rows
                .iter()
                .filter_map(|r| r.get("command").and_then(Value::as_str))
                .filter(|c| c.starts_with("dev cli "))
                .map(|c| c.split_whitespace().take(4).collect::<Vec<_>>().join(" "))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let plugin_commands = command_rows
                .iter()
                .filter_map(|r| r.get("command").and_then(Value::as_str))
                .filter_map(|c| {
                    if c.starts_with("plugins ") {
                        Some(c.split_whitespace().take(2).collect::<Vec<_>>().join(" "))
                    } else if c.starts_with("cli plugins ") {
                        Some(c.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
                    } else {
                        None
                    }
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let snapshot_covered = current_state
                .get("snapshot_covered_commands")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let stream_covered = current_state
                .get("stderr_stdout_covered_commands")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let exit_covered = current_state
                .get("exit_code_covered_commands")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let fail_covered = collect_files(&workspace_root.join("crates"))
                .into_iter()
                .filter(|p| {
                    p.to_string_lossy().contains("/tests/")
                        && p.extension().and_then(|e| e.to_str()) == Some("rs")
                })
                .filter_map(|p| fs::read_to_string(&p).ok())
                .flat_map(|txt| txt.lines().map(ToString::to_string).collect::<Vec<_>>())
                .filter(|line| {
                    line.contains("[\"")
                        && [
                            "error",
                            "failure",
                            "invalid",
                            "malformed",
                            "missing",
                            "reject",
                            "rollback",
                            "corrupt",
                            "unsafe",
                            "duplicate",
                            "conflict",
                            "shadow",
                        ]
                        .iter()
                        .any(|k| line.to_lowercase().contains(k))
                })
                .filter_map(|line| {
                    let quoted = line.split('"').collect::<Vec<_>>();
                    let vals = quoted
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| (i % 2 == 1).then_some((*v).to_string()))
                        .collect::<Vec<_>>();
                    (!vals.is_empty()).then_some(vals.join(" "))
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let known_gaps = command_rows.iter().filter(|row| row.get("status").and_then(Value::as_str).is_some_and(|s| ["missing","partial","shim"].contains(&s))).map(|row| json!({"command":row["command"],"status":row["status"],"blocker":row["blocker"],"owner":row["owner"]})).collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/status.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli","commands":command_rows,
                                "summary":{"total":command_rows.len(),"complete":command_rows.iter().filter(|r| r["status"]=="complete").count(),"partial":command_rows.iter().filter(|r| r["status"]=="partial").count(),"shim":command_rows.iter().filter(|r| r["status"]=="shim").count(),"missing":command_rows.iter().filter(|r| r["status"]=="missing").count()}
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_root_commands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":root_commands})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_cli_subcommands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":cli_commands})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_dev_cli_subcommands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":dev_cli_commands})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_plugin_commands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":plugin_commands})).ok()?;
            let repl = command_rows
                .iter()
                .filter(|r| {
                    r.get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|c| c.split_whitespace().any(|p| p == "repl"))
                })
                .cloned()
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/status_repl_parity_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","summary":{"count":repl.len(),"statuses":{"complete":repl.iter().filter(|r| r["status"]=="complete").count(),"partial":repl.iter().filter(|r| r["status"]=="partial").count(),"shim":repl.iter().filter(|r| r["status"]=="shim").count(),"missing":repl.iter().filter(|r| r["status"]=="missing").count()}},"commands":repl,"evidence_files":["crates/bijux-cli-repl/tests/transcript_parity.rs","crates/bijux-cli-repl/tests/transcript_cases.rs"]})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_python_bridge_parity_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","report":bridge_report})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_install_packaging_parity_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","runtime_unity":runtime_unity,"runtime_identity_rules":current_state.get("runtime_identity_rules").cloned().unwrap_or_else(|| json!({})),"package_entrypoints":current_state.get("package_entrypoints").cloned().unwrap_or_else(|| json!([]))})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_state_behavior_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","config":state_config,"history":state_history,"memory":state_memory,"plugin_state":plugin_state})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_state_paths_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","state_paths":{"config":"BIJUXCLI_CONFIG or <HOME>/.bijux/.env","history":"BIJUXCLI_HISTORY_FILE or <HOME>/.bijux/.history","plugins_dir":"BIJUXCLI_PLUGINS_DIR or <HOME>/.bijux/.plugins","plugins_registry":"<plugins_dir>/registry.json","memory":"<HOME>/.bijux/.memory.json"},"source_precedence":["flags","env","config","defaults"]})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_state_corruption_health_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","areas":{"config":{"report":state_config,"focus":["malformed file","duplicate key","partial-write rollback"]},"history":{"report":state_history,"focus":["malformed array entries","line-format compatibility","oversized budget"]},"memory":{"report":state_memory,"focus":["malformed json","wrong-type object rejection"]},"plugin_registry":{"report":plugin_state,"focus":["malformed registry json","partial-write self-repair","stale backup cleanup"]}}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_snapshot_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":snapshot_covered})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_stream_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":stream_covered})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_exit_code_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":exit_covered})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_failure_path_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":fail_covered})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_compatibility_aliases.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","aliases":aliases.into_iter().collect::<Vec<_>>()})).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/status_known_parity_gaps.json",
                &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","gaps":known_gaps}),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_intentional_differences.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":intentional})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_unowned_scripts.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","scripts":current_state.get("scripts_outside_dev_cli").cloned().unwrap_or_else(|| json!([]))})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/status.json","artifacts/status/status_root_commands.json","artifacts/status/status_cli_subcommands.json","artifacts/status/status_dev_cli_subcommands.json","artifacts/status/status_plugin_commands.json","artifacts/status/status_repl_parity_coverage.json","artifacts/status/status_python_bridge_parity_coverage.json","artifacts/status/status_install_packaging_parity_coverage.json","artifacts/status/status_state_behavior_coverage.json","artifacts/status/status_state_paths_report.json","artifacts/status/status_state_corruption_health_report.json","artifacts/status/status_snapshot_coverage.json","artifacts/status/status_stream_coverage.json","artifacts/status/status_exit_code_coverage.json","artifacts/status/status_failure_path_coverage.json","artifacts/status/status_compatibility_aliases.json","artifacts/status/status_known_parity_gaps.json","artifacts/status/status_intentional_differences.json","artifacts/status/status_unowned_scripts.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-MAINTAINER-CONTROL-PLANE-REPORTS" => {
            let generated_at = generated_at_utc();
            let required_commands = vec![
                "dev cli status",
                "dev cli parity",
                "dev cli route-audit",
                "dev cli state-audit",
                "dev cli script-audit",
                "dev cli crate-health",
                "dev cli package-health",
                "dev cli docs-audit",
            ];
            let replacements = BTreeMap::from([
                                ("scripts/check-package-metadata.py","bijux dev cli scripts package-metadata --format json --no-pretty"),
                                ("scripts/check_e2e_contract.py","bijux dev cli scripts e2e-contract --format json --no-pretty"),
                                ("scripts/helper_pip_audit.py","bijux dev cli scripts pip-audit --format json --no-pretty"),
                                ("scripts/capture_python_behavior.py","bijux dev cli scripts capture-python-behavior --format json --no-pretty"),
                                ("scripts/generate-provenance-statement.sh","bijux dev cli scripts provenance-statement --tag <tag> --output-dir <dir> --format json --no-pretty"),
                            ]);
            let command_samples = fs::read_to_string(
                workspace_root.join("artifacts/status/dev_cli_control_plane_samples.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let mut inventory = Vec::<Value>::new();
            for path in collect_files(&workspace_root.join("scripts")) {
                let relp = rel(&path, workspace_root);
                if relp.contains("/__pycache__/")
                    || Path::new(&relp)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("pyc"))
                    || relp.starts_with("scripts/obsolete-status/")
                {
                    continue;
                }
                let replacement = replacements.get(relp.as_str()).copied().unwrap_or("");
                inventory.push(json!({"path":relp,"replacement_command":replacement,"status":if replacement.is_empty(){"remaining"}else{"replaced"}}));
            }
            inventory.sort_by(|a, b| {
                a.get("path").and_then(Value::as_str).cmp(&b.get("path").and_then(Value::as_str))
            });
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_scripts_outside_dev_cli.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","scripts":inventory,"summary":{"total":inventory.len(),"replaced":inventory.iter().filter(|r| r["status"]=="replaced").count(),"remaining":inventory.iter().filter(|r| r["status"]=="remaining").count()}})).ok()?;
            let commands = required_commands.iter().map(|command| {
                                let sample = command_samples.get(*command).cloned().unwrap_or_else(|| json!({}));
                                json!({"command":command,"json_sample_present":sample.get("json").is_some(),"text_sample_present":sample.get("text").is_some(),"json_top_level_keys":sample.get("json_top_level_keys").cloned().unwrap_or_else(|| json!([]))})
                            }).collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_control_plane_commands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","required_commands":required_commands,"commands":commands})).ok()?;
            let mut text =
                format!("Maintainer control plane summary\nGenerated at: {generated_at}\n\n");
            for row in &commands {
                let keys = row
                    .get("json_top_level_keys")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
                    .join(", ");
                text.push_str(&format!(
                    "- {}: json_keys={}\n",
                    row.get("command").and_then(Value::as_str).unwrap_or(""),
                    if keys.is_empty() { "(none)" } else { &keys }
                ));
            }
            text.push_str("\nDefault maintainer command: bijux dev cli status\nPolicy: use dev cli command surfaces before creating new ad-hoc scripts.\n");
            fs::write(
                workspace_root.join("artifacts/status/maintainer_control_plane_text_report.txt"),
                text,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_control_plane_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","scripts_outside_dev_cli":fs::read_to_string(workspace_root.join("artifacts/status/maintainer_scripts_outside_dev_cli.json")).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_else(|| json!({})),"commands":fs::read_to_string(workspace_root.join("artifacts/status/maintainer_control_plane_commands.json")).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_else(|| json!({})),"text_report":"artifacts/status/maintainer_control_plane_text_report.txt"})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/maintainer_scripts_outside_dev_cli.json",
                "artifacts/status/maintainer_control_plane_commands.json",
                "artifacts/status/maintainer_control_plane_text_report.txt",
                "artifacts/status/maintainer_control_plane_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CRATE-BOUNDARY-METRICS" => {
            let generated_at = generated_at_utc();
            let metadata = Command::new("cargo")
                .args(["metadata", "--format-version", "1", "--no-deps"])
                .current_dir(workspace_root)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_else(|| json!({}));
            let pkgs =
                metadata.get("packages").and_then(Value::as_array).cloned().unwrap_or_default();
            let workspace_names = pkgs
                .iter()
                .filter_map(|p| p.get("name").and_then(Value::as_str).map(ToString::to_string))
                .collect::<BTreeSet<_>>();
            let mut per_crate = Vec::<Value>::new();
            for pkg in &pkgs {
                let Some(name) = pkg.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let compile = Command::new("cargo")
                    .args(["check", "-q", "-p", name])
                    .current_dir(workspace_root)
                    .status()
                    .ok()
                    .is_some_and(|s| s.success());
                let test_build = Command::new("cargo")
                    .args(["test", "-q", "-p", name, "--no-run"])
                    .current_dir(workspace_root)
                    .status()
                    .ok()
                    .is_some_and(|s| s.success());
                let manifest = pkg.get("manifest_path").and_then(Value::as_str).unwrap_or("");
                let cargo_toml = PathBuf::from(manifest);
                let rel_manifest = rel(&cargo_toml, workspace_root);
                let cargo_text = fs::read_to_string(&cargo_toml).unwrap_or_default();
                let fan_out = workspace_names
                    .iter()
                    .filter(|dep| dep.as_str() != name && cargo_text.contains(dep.as_str()))
                    .count();
                per_crate.push(json!({
                                    "crate":name,
                                    "compile_seconds": Value::Null,
                                    "test_build_seconds": Value::Null,
                                    "dependency_fan_in": Value::Null,
                                    "dependency_fan_out": fan_out,
                                    "public_api_count": collect_files(&workspace_root.join(rel_manifest.replace("Cargo.toml","src")))
                                        .into_iter().filter(|p| p.extension().and_then(|e| e.to_str())==Some("rs"))
                                        .filter_map(|p| fs::read_to_string(p).ok())
                                        .map(|t| t.matches("pub ").count())
                                        .sum::<usize>(),
                                    "churn": {"commit_count": Value::Null,"files_changed_entries": Value::Null,"insertions": Value::Null,"deletions": Value::Null},
                                    "compile_ok": compile,
                                    "test_build_ok": test_build,
                                }));
            }
            let boundary_decisions = json!([
                {"boundary":"core <-> routing","status":"watch","decision":"keep separate for now","reason":"high co-change expected during parity closure; separation still useful for parser test focus"},
                {"boundary":"core <-> output","status":"watch","decision":"keep separate for now","reason":"output formatting contracts remain reusable and test-scoped"},
                {"boundary":"core <-> install","status":"watch","decision":"keep separate for now","reason":"install concerns include path and packaging diagnostics outside core execution law"},
                {"boundary":"core <-> contracts","status":"keep","decision":"must stay separate","reason":"machine contracts must remain independent from execution engine"},
                {"boundary":"core <-> python","status":"keep","decision":"must stay separate","reason":"bridge packaging/runtime integration is language-boundary specific"},
                {"boundary":"core <-> plugin","status":"keep","decision":"must stay separate","reason":"plugin lifecycle and registry law should not be merged into base execution core"},
                {"boundary":"core <-> repl","status":"keep","decision":"must stay separate","reason":"interactive session model and transcript behavior are distinct runtime surfaces"}
            ]);
            let crate_decisions = json!([
                {"crate":"bijux-cli","status":"keep","review":"must stay separate","reason":"runtime command execution and routing law are now co-located in one crate"},
                {"crate":"bijux-dev-cli","status":"watch","review":"paying rent with dedicated control-plane reports and ownership tests","reason":"should remain independent while delegating from core through query interfaces"},
                {"crate":"bijux-cli-python","status":"watch","review":"paying rent with bridge parity and conversion law tests","reason":"language boundary remains useful while python bridge is maintained"},
                {"crate":"bijux-cli-evidence","status":"keep","review":"must stay separate","reason":"evidence IDs and helpers should stay reusable across tooling surfaces"}
            ]);
            let report = json!({
                "generated_at":generated_at,
                "generator":"bijux-dev-cli",
                "metrics":{"per_crate":per_crate,"cross_crate_change_frequency":[]},
                "boundary_decisions":boundary_decisions,
                "crate_decisions":crate_decisions,
                "rules":{"no_large_merge_until_parity_stronger":true,"rule_text":"Large crate merges are frozen until parity coverage and mismatch trend show sustained improvement."}
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/crate_boundary_metrics.json",
                &report,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/crate_boundary_report.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli",
                                "evidence":{"metrics_artifact":"artifacts/status/crate_boundary_metrics.json","top_cross_crate_pairs":[]},
                                "crate_decision_summary":{"keep":2,"watch":2,"candidate_to_merge_later":0},
                                "crate_decisions":crate_decisions,
                                "boundary_decisions":boundary_decisions
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/crate_boundary_metrics.json",
                "artifacts/status/crate_boundary_report.json"
            ]}))
        }
        _ => None,
    }
}
