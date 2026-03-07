use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScenarioRegistryEntry {
    pub(super) id: String,
    pub(super) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScenarioRegistry {
    pub(super) entries: Vec<ScenarioRegistryEntry>,
}

pub(super) fn load_scenario_registry(root: &Path) -> Result<ScenarioRegistry, String> {
    let registry_path = root.join("evidence/perf/scenario_registry.json");
    let payload = fs::read_to_string(&registry_path)
        .map_err(|err| format!("read scenario registry {}: {err}", registry_path.display()))?;
    let value: Value =
        serde_json::from_str(&payload).map_err(|err| format!("parse scenario registry: {err}"))?;

    let entries = value["entries"]
        .as_array()
        .ok_or_else(|| "scenario registry entries must be an array".to_string())?;

    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let id = entry["id"]
            .as_str()
            .ok_or_else(|| "scenario registry entry missing id".to_string())?
            .to_string();
        let path = entry["path"]
            .as_str()
            .ok_or_else(|| format!("scenario registry entry `{id}` missing path"))?
            .to_string();
        out.push(ScenarioRegistryEntry { id, path });
    }

    Ok(ScenarioRegistry { entries: out })
}

pub(super) fn verify_scenario_registry(root: &Path) -> Result<(), String> {
    let registry = load_scenario_registry(root)?;
    let required_ids = [
        "tiny-canonical",
        "wide-canonical",
        "deep-canonical",
        "tenk-nodes-canonical",
        "large-artifact-canonical",
        "cache-heavy-canonical",
        "failure-injection-canonical",
        "replay-canonical",
        "diff-canonical",
        "portability-canonical",
        "determinism-score",
        "replay-fidelity-score",
        "explainability-quality",
        "artifact-lineage-completeness",
        "portability-success-rate",
    ];

    let mut ids = BTreeSet::new();
    for entry in &registry.entries {
        if !ids.insert(entry.id.clone()) {
            return Err(format!("duplicate scenario registry id: {}", entry.id));
        }
        if !root.join(&entry.path).exists() {
            return Err(format!(
                "scenario registry points to missing scenario file: {}",
                entry.path
            ));
        }
    }

    for required in required_ids {
        if !ids.contains(required) {
            return Err(format!(
                "scenario registry missing required benchmark scenario id: {required}"
            ));
        }
    }

    Ok(())
}
