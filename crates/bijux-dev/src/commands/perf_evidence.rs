use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use super::benchmark_harness::verify_scenario_registry;

fn repo_root() -> Result<PathBuf, String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir.join("../.."))
}

fn load_perf_metadata(root: &Path) -> Result<Value, String> {
    let metadata_path = root.join("evidence/perf/metadata.json");
    serde_json::from_str(&fs::read_to_string(&metadata_path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

fn collect_all_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            collect_all_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn collect_perf_scenario_files(root: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    collect_all_files(&root.join("evidence/perf/scenarios"), &mut files)?;
    let mut rels = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if rel.ends_with(".json") {
            rels.push(rel);
        }
    }
    rels.sort();
    Ok(rels)
}

pub(super) fn run_perf_evidence_policy_verify() -> Result<(), String> {
    let root = repo_root()?;
    verify_scenario_registry(&root)?;
    let metadata = load_perf_metadata(&root)?;
    let contract_reference = metadata["contract_reference"]
        .as_str()
        .ok_or_else(|| "perf metadata contract_reference must be a string".to_string())?;
    if !root.join(contract_reference).exists() {
        return Err(format!("perf metadata references missing contract: {contract_reference}"));
    }
    let scenarios = metadata["scenarios"]
        .as_object()
        .ok_or_else(|| "perf metadata scenarios must be an object".to_string())?;
    let release_set = metadata["release_relevant_set"]
        .as_array()
        .ok_or_else(|| "perf metadata release_relevant_set must be an array".to_string())?;

    if release_set.is_empty() {
        return Err("perf metadata release_relevant_set must not be empty".to_string());
    }

    let scenario_files = collect_perf_scenario_files(&root)?;
    for rel in &scenario_files {
        if !scenarios.contains_key(rel) {
            return Err(format!("perf scenario file is missing metadata classification: {rel}"));
        }
    }
    for rel in scenarios.keys() {
        if !root.join(rel).exists() {
            return Err(format!("perf metadata references missing scenario file: {rel}"));
        }
    }

    for (rel, entry) in scenarios {
        let scenario_class = entry
            .get("scenario_class")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("perf scenario missing scenario_class: {rel}"))?;
        if !["core", "advisory", "experimental"].contains(&scenario_class) {
            return Err(format!(
                "perf scenario has invalid scenario_class `{scenario_class}`: {rel}"
            ));
        }
        let release_blocking = entry
            .get("release_blocking")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("perf scenario missing release_blocking boolean: {rel}"))?;
        let threshold_reference = entry
            .get("threshold_reference")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("perf scenario missing threshold_reference: {rel}"))?;
        let scenario_contract_reference =
            entry
                .get("contract_reference")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("perf scenario missing contract_reference: {rel}"))?;

        if !root.join(scenario_contract_reference).exists() {
            return Err(format!(
                "perf scenario references missing contract: {rel} -> {scenario_contract_reference}"
            ));
        }
        if scenario_contract_reference != contract_reference {
            return Err(format!(
                "perf scenario contract_reference must match perf metadata contract_reference: {rel}"
            ));
        }

        if release_blocking {
            if threshold_reference.trim().is_empty() {
                return Err(format!(
                    "release-blocking perf scenario must declare threshold_reference: {rel}"
                ));
            }
            if scenario_class != "core" {
                return Err(format!(
                    "release-blocking perf scenario must be classified as core: {rel}"
                ));
            }
        }
        if (scenario_class == "advisory" || scenario_class == "experimental") && release_blocking {
            return Err(format!(
                "advisory or experimental perf scenario cannot be release_blocking: {rel}"
            ));
        }
    }

    for item in release_set {
        let rel =
            item.as_str().ok_or_else(|| "release_relevant_set entry must be string".to_string())?;
        let entry = scenarios
            .get(rel)
            .ok_or_else(|| format!("release_relevant_set references unknown scenario: {rel}"))?;
        let release_blocking = entry["release_blocking"].as_bool().unwrap_or(false);
        if !release_blocking {
            return Err(format!("release_relevant_set scenario must be release_blocking: {rel}"));
        }
    }
    Ok(())
}

pub(super) fn run_perf_evidence_summary() -> Result<(), String> {
    let root = repo_root()?;
    let metadata = load_perf_metadata(&root)?;
    run_perf_evidence_policy_verify()?;

    let scenarios = metadata["scenarios"]
        .as_object()
        .ok_or_else(|| "perf metadata scenarios must be an object".to_string())?;
    let mut core = Vec::new();
    let mut advisory = Vec::new();
    let mut experimental = Vec::new();
    for (rel, entry) in scenarios {
        let scenario_class = entry["scenario_class"].as_str().unwrap_or("");
        let release_blocking = entry["release_blocking"].as_bool().unwrap_or(false);
        let record = json!({
            "path": rel,
            "workload_class": entry["workload_class"].as_str().unwrap_or(""),
            "release_blocking": release_blocking,
            "threshold_reference": entry["threshold_reference"].as_str().unwrap_or("")
        });
        match scenario_class {
            "core" => core.push(record),
            "advisory" => advisory.push(record),
            _ => experimental.push(record),
        }
    }

    let payload = json!({
        "contract_reference": metadata["contract_reference"],
        "release_relevant_set": metadata["release_relevant_set"],
        "core": core,
        "advisory": advisory,
        "experimental": experimental,
        "obsolete_candidates": metadata["obsolete_candidates"]
    });
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_perf_release_set() -> Result<(), String> {
    let root = repo_root()?;
    let metadata = load_perf_metadata(&root)?;
    run_perf_evidence_policy_verify()?;
    let release_set = metadata["release_relevant_set"]
        .as_array()
        .ok_or_else(|| "perf metadata release_relevant_set must be an array".to_string())?;
    let scenarios = metadata["scenarios"]
        .as_object()
        .ok_or_else(|| "perf metadata scenarios must be an object".to_string())?;
    let mut rows = Vec::new();
    for rel in release_set {
        let rel =
            rel.as_str().ok_or_else(|| "release_relevant_set entry must be string".to_string())?;
        let entry = scenarios
            .get(rel)
            .ok_or_else(|| format!("release_relevant_set references unknown scenario: {rel}"))?;
        rows.push(json!({
            "path": rel,
            "workload_class": entry["workload_class"].as_str().unwrap_or(""),
            "threshold_reference": entry["threshold_reference"].as_str().unwrap_or(""),
            "threshold_owner": entry["threshold_owner"].as_str().unwrap_or("")
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "release_set": rows
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_performance_evidence_guard() -> Result<(), String> {
    let root = repo_root()?;
    let metadata = load_perf_metadata(&root)?;
    let contract_reference = metadata["contract_reference"]
        .as_str()
        .ok_or_else(|| "perf metadata contract_reference must be a string".to_string())?;
    for rel in [
        "configs/dag/schema/benchmarks/benchmark_report.schema.json",
        "evidence/perf/baselines/regression_thresholds.json",
        "evidence/perf/metadata.json",
        "evidence/reports/perf_obsolete_candidates.md",
        "crates/bijux-dev/tests/perf_evidence_contracts.rs",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing performance evidence artifact: {rel}"));
        }
    }
    if !root.join(contract_reference).exists() {
        return Err(format!("missing performance evidence artifact: {contract_reference}"));
    }
    run_perf_evidence_policy_verify()
}

pub(super) fn run_performance_evidence_report() -> Result<(), String> {
    run_perf_evidence_summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_root_points_to_workspace_with_evidence_dir() {
        let root = repo_root().expect("repo root");
        assert!(root.join("evidence").is_dir());
        assert!(root.join("Cargo.toml").exists());
    }

    #[test]
    fn perf_metadata_has_required_top_level_fields() {
        let root = repo_root().expect("repo root");
        let metadata = load_perf_metadata(&root).expect("perf metadata");
        assert!(metadata.get("contract_reference").is_some());
        assert!(metadata.get("scenarios").and_then(Value::as_object).is_some());
        assert!(metadata.get("release_relevant_set").and_then(Value::as_array).is_some());
    }

    #[test]
    fn perf_scenario_file_collector_finds_json_assets() {
        let root = repo_root().expect("repo root");
        let scenarios = collect_perf_scenario_files(&root).expect("scenario files");
        assert!(!scenarios.is_empty(), "expected perf scenario fixtures");
        assert!(scenarios.iter().all(|path| path.ends_with(".json")));
    }
}
