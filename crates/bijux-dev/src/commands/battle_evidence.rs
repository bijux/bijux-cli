use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::{collect_files_with_extension, repo_root};

fn load_battle_policy(root: &Path) -> Result<Value, String> {
    let policy_path = root.join("configs/dag/policy/battle_trust_properties.json");
    serde_json::from_str(&fs::read_to_string(policy_path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

pub(super) fn load_battle_scenario_records(root: &Path) -> Result<Vec<(String, String)>, String> {
    let workflows_root = root.join("evidence/battle/workflows");
    let mut files = Vec::new();
    collect_files_with_extension(&workflows_root, "json", &mut files)?;
    files.sort();

    let mut records = Vec::new();
    for path in files {
        let raw = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let doc: Value = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
        let scenario_id = doc
            .get("scenario")
            .and_then(Value::as_str)
            .or_else(|| doc.get("scenario_id").and_then(Value::as_str))
            .or_else(|| path.file_stem().and_then(|stem| stem.to_str()))
            .ok_or_else(|| format!("unable to determine scenario id for {}", path.display()))?
            .to_string();
        let rel = path
            .strip_prefix(root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        records.push((scenario_id, rel));
    }

    Ok(records)
}

pub(super) fn run_battle_scenarios_report() -> Result<(), String> {
    let root = repo_root()?;
    let records = load_battle_scenario_records(&root)?;
    let policy = load_battle_policy(&root)?;
    let scenario_trust_map = policy
        .get("scenario_trust_map")
        .and_then(Value::as_object)
        .ok_or_else(|| "battle trust policy missing scenario_trust_map".to_string())?;
    let mut rows = Vec::new();
    for (scenario, path) in records {
        let mapped = scenario_trust_map
            .get(&scenario)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        rows.push(json!({
            "scenario": scenario,
            "path": path,
            "trust_properties": mapped,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": "1",
            "battle_scenarios": rows
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_battle_scenarios_by_trust_report() -> Result<(), String> {
    let root = repo_root()?;
    let records = load_battle_scenario_records(&root)?;
    let policy = load_battle_policy(&root)?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let scenario_trust_map = policy
        .get("scenario_trust_map")
        .and_then(Value::as_object)
        .ok_or_else(|| "battle trust policy missing scenario_trust_map".to_string())?;

    let path_map: BTreeMap<String, String> = records.into_iter().collect();
    let mut by_trust = Vec::new();
    for trust in trust_properties {
        let trust_id = trust
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "trust property missing id".to_string())?;
        let mut scenarios = Vec::new();
        for (scenario, mapped) in scenario_trust_map {
            let mapped = mapped
                .as_array()
                .ok_or_else(|| format!("scenario `{scenario}` trust mapping must be array"))?;
            if mapped.iter().any(|value| value.as_str() == Some(trust_id)) {
                scenarios.push(json!({
                    "scenario": scenario,
                    "path": path_map.get(scenario).cloned().unwrap_or_default()
                }));
            }
        }
        scenarios.sort_by(|a, b| {
            a["scenario"]
                .as_str()
                .unwrap_or("")
                .cmp(b["scenario"].as_str().unwrap_or(""))
        });
        by_trust.push(json!({
            "trust_property_id": trust_id,
            "scenario_count": scenarios.len(),
            "scenarios": scenarios
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": "1",
            "grouped_by": "trust_property",
            "rows": by_trust
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_battle_trust_by_scenario_report() -> Result<(), String> {
    let root = repo_root()?;
    let records = load_battle_scenario_records(&root)?;
    let policy = load_battle_policy(&root)?;
    let scenario_trust_map = policy
        .get("scenario_trust_map")
        .and_then(Value::as_object)
        .ok_or_else(|| "battle trust policy missing scenario_trust_map".to_string())?;
    let mut rows = Vec::new();
    for (scenario, path) in records {
        let mapped = scenario_trust_map
            .get(&scenario)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        rows.push(json!({
            "scenario": scenario,
            "path": path,
            "trust_properties": mapped
        }));
    }
    rows.sort_by(|a, b| {
        a["scenario"]
            .as_str()
            .unwrap_or("")
            .cmp(b["scenario"].as_str().unwrap_or(""))
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": "1",
            "grouped_by": "scenario",
            "rows": rows
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_battle_scenario_mapping_validate() -> Result<(), String> {
    let root = repo_root()?;
    let policy = load_battle_policy(&root)?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let trust_ids: BTreeSet<&str> = trust_properties
        .iter()
        .filter_map(|entry| entry.get("id").and_then(Value::as_str))
        .collect();
    let top_trust: BTreeSet<&str> = policy
        .get("release_top_trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing release_top_trust_properties".to_string())?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let scenario_trust_map = policy
        .get("scenario_trust_map")
        .and_then(Value::as_object)
        .ok_or_else(|| "battle trust policy missing scenario_trust_map".to_string())?;
    let records = load_battle_scenario_records(&root)?;

    for (scenario, _) in &records {
        let mapped = scenario_trust_map
            .get(scenario)
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("battle scenario `{scenario}` is missing trust-property mapping")
            })?;
        if mapped.is_empty() {
            return Err(format!(
                "battle scenario `{scenario}` must map to at least one trust property"
            ));
        }
        if mapped.len() > 3 {
            return Err(format!(
                "battle scenario `{scenario}` maps too many trust properties ({}); split oversized scenario proof",
                mapped.len()
            ));
        }
        for trust in mapped {
            let trust = trust
                .as_str()
                .ok_or_else(|| format!("non-string trust property mapping for `{scenario}`"))?;
            if !trust_ids.contains(trust) {
                return Err(format!(
                    "battle scenario `{scenario}` maps unknown trust property `{trust}`"
                ));
            }
        }
    }

    let known: BTreeSet<&str> = records
        .iter()
        .map(|(scenario, _)| scenario.as_str())
        .collect();
    for mapped in scenario_trust_map.keys() {
        if !known.contains(mapped.as_str()) {
            return Err(format!("battle trust map has orphan scenario `{mapped}`"));
        }
    }

    for trust in &top_trust {
        let covered = scenario_trust_map.values().any(|mapped| {
            mapped
                .as_array()
                .map(|array| array.iter().any(|value| value.as_str() == Some(*trust)))
                .unwrap_or(false)
        });
        if !covered {
            return Err(format!(
                "top trust property `{trust}` has no mapped battle scenario"
            ));
        }
    }

    let registry_path = root.join("evidence/battle/registries/scenario_registry.json");
    if !registry_path.exists() {
        return Err(format!(
            "missing battle scenario registry: {}",
            registry_path.display()
        ));
    }
    let registry: Value =
        serde_json::from_str(&fs::read_to_string(&registry_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let registry_entries = registry
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle scenario registry must contain entries array".to_string())?;
    for entry in registry_entries {
        let scenario = entry
            .get("scenario")
            .and_then(Value::as_str)
            .ok_or_else(|| "battle scenario registry entry missing scenario".to_string())?;
        let path = entry
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "battle scenario registry entry missing path".to_string())?;
        if !known.contains(scenario) {
            return Err(format!(
                "battle scenario registry entry `{scenario}` points to non-existent scenario"
            ));
        }
        if !root.join(path).exists() {
            return Err(format!(
                "battle scenario registry path missing for `{scenario}`: {path}"
            ));
        }
    }

    let metadata_path = root.join("evidence/battle/metadata.json");
    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let scenarios = metadata
        .get("scenarios")
        .and_then(Value::as_object)
        .ok_or_else(|| "battle metadata must contain scenarios object".to_string())?;
    let required_scenarios = policy
        .get("required_scenarios")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing required_scenarios".to_string())?;
    let required_set: BTreeSet<&str> = required_scenarios
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for scenario in known {
        let metadata_entry = scenarios.get(scenario).ok_or_else(|| {
            format!("battle metadata missing scenario entry `{scenario}` in evidence/battle/metadata.json")
        })?;
        let release_blocking = metadata_entry
            .get("release_blocking")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                format!("battle metadata `{scenario}` missing release_blocking boolean")
            })?;
        if release_blocking {
            if !required_set.contains(scenario) {
                return Err(format!(
                    "release-blocking battle scenario `{scenario}` must appear in required_scenarios"
                ));
            }
            let mapped = scenario_trust_map
                .get(scenario)
                .and_then(Value::as_array)
                .ok_or_else(|| format!("battle scenario `{scenario}` missing trust mapping"))?;
            let protects_top = mapped
                .iter()
                .filter_map(Value::as_str)
                .any(|trust| top_trust.contains(trust));
            if !protects_top {
                return Err(format!(
                    "release-blocking battle scenario `{scenario}` must protect at least one top trust property"
                ));
            }
        }
    }

    println!("battle scenario mapping validation passed");
    Ok(())
}

pub(super) fn run_battle_coverage_report(
    gaps_out: &Path,
    overloaded_out: &Path,
) -> Result<(), String> {
    let root = repo_root()?;
    let policy = load_battle_policy(&root)?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let scenario_trust_map = policy
        .get("scenario_trust_map")
        .and_then(Value::as_object)
        .ok_or_else(|| "battle trust policy missing scenario_trust_map".to_string())?;

    let mut coverage_gaps = Vec::new();
    for trust in trust_properties {
        let trust_id = trust
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "trust property missing id".to_string())?;
        let covered = scenario_trust_map.values().any(|mapped| {
            mapped
                .as_array()
                .map(|array| array.iter().any(|value| value.as_str() == Some(trust_id)))
                .unwrap_or(false)
        });
        if !covered {
            coverage_gaps.push(trust_id.to_string());
        }
    }

    let mut overloaded = Vec::new();
    for (scenario, mapped) in scenario_trust_map {
        let count = mapped
            .as_array()
            .map(|values| values.len())
            .ok_or_else(|| format!("scenario trust mapping for `{scenario}` must be array"))?;
        if count > 3 {
            overloaded.push((scenario.clone(), count));
        }
    }
    overloaded.sort_by(|a, b| a.0.cmp(&b.0));

    let mut gaps_report = String::new();
    gaps_report.push_str("# Battle Coverage Gaps\n\n");
    if coverage_gaps.is_empty() {
        gaps_report.push_str("No battle trust-property coverage gaps detected.\n");
    } else {
        gaps_report.push_str("Uncovered trust properties:\n");
        for trust in &coverage_gaps {
            gaps_report.push_str(&format!("- `{trust}`\n"));
        }
    }

    let mut overloaded_report = String::new();
    overloaded_report.push_str("# Overloaded Battle Scenarios\n\n");
    if overloaded.is_empty() {
        overloaded_report.push_str("No overloaded battle scenarios detected.\n");
    } else {
        overloaded_report.push_str("Scenarios mapping more than three trust properties:\n");
        for (scenario, count) in &overloaded {
            overloaded_report
                .push_str(&format!("- `{scenario}` maps `{count}` trust properties\n"));
        }
    }

    let gaps_path = if gaps_out.is_absolute() {
        PathBuf::from(gaps_out)
    } else {
        root.join(gaps_out)
    };
    if let Some(parent) = gaps_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(&gaps_path, gaps_report).map_err(|err| err.to_string())?;

    let overloaded_path = if overloaded_out.is_absolute() {
        PathBuf::from(overloaded_out)
    } else {
        root.join(overloaded_out)
    };
    if let Some(parent) = overloaded_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(&overloaded_path, overloaded_report).map_err(|err| err.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::load_battle_scenario_records;
    use std::fs;

    #[test]
    fn scenario_records_use_declared_scenario_field() {
        let temp = tempfile::tempdir().expect("tempdir");
        let workflow = temp.path().join("evidence/battle/workflows");
        fs::create_dir_all(&workflow).expect("create workflow dir");
        fs::write(
            workflow.join("alpha.json"),
            "{ \"scenario\": \"battle-alpha\", \"detail\": \"ok\" }",
        )
        .expect("write workflow");

        let records = load_battle_scenario_records(temp.path()).expect("load records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].0, "battle-alpha");
        assert_eq!(records[0].1, "evidence/battle/workflows/alpha.json");
    }
}
