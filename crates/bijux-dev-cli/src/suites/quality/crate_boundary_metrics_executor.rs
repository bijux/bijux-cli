#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
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
