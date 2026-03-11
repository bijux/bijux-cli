#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

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
            let release_bin = file_info(&workspace_root.join("target/release/bijux"));
            let debug_bin = file_info(&workspace_root.join("target/debug/bijux"));
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
            let maintenance_audit = read("artifacts/status/maintenance_gap_behaviors.json");
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
            write_status_artifact_json(workspace_root, "artifacts/status/release_status_manifest.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","scope":"release status manifest","status":if missing.is_empty(){"ready"}else{"blocked"},"coverage_ids":[189,200],"checks":{"missing_evidence":missing,"parity_partial_count":partial.len(),"parity_missing_count":missing_cmd.len(),"stale_maintenance_outside_dev_cli":maintenance_audit.get("maintenance").and_then(Value::as_array).map_or(0,Vec::len),"docs_markdown_count":docs_audit.get("markdown_count").and_then(Value::as_i64).unwrap_or(0),"weak_tests_count":weak_tests.len()},"review_steps":["review intentionally different behaviors","review unresolved partial commands","review stale maintenance outside dev cli","review stale docs from docs audit","review weak tests from test audit","review release evidence bundle before release candidate decision"],"next_work_input":"Use release_evidence_bundle.json and release_truth_report.json as the first input for next prioritization.","status_discussion_policy":"status claims are invalid unless backed by artifacts in this manifest"})).ok()?;
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
        _ => None,
    }
}
