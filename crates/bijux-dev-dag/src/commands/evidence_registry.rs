use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::{collect_all_files, repo_root};

pub(super) fn run_evidence_ledger_normalize(check: bool) -> Result<(), String> {
    let root = repo_root()?;
    let ledger_path = root.join("evidence/ownership/evidence_ledger.json");
    let mut ledger: Value =
        serde_json::from_str(&fs::read_to_string(&ledger_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let entries = ledger["entries"]
        .as_array_mut()
        .ok_or_else(|| "evidence ledger entries must be an array".to_string())?;
    entries.sort_by(|a, b| {
        let kind_a = a["kind"].as_str().unwrap_or("");
        let kind_b = b["kind"].as_str().unwrap_or("");
        let id_a = a["id"].as_str().unwrap_or("");
        let id_b = b["id"].as_str().unwrap_or("");
        (kind_a, id_a).cmp(&(kind_b, id_b))
    });
    let normalized = format!(
        "{}\n",
        serde_json::to_string_pretty(&ledger).map_err(|err| err.to_string())?
    );
    let current = fs::read_to_string(&ledger_path).map_err(|err| err.to_string())?;
    if check {
        if current != normalized {
            return Err(
                "evidence ledger is not normalized; run `bijux-dev-dag repo evidence-ledger-normalize`"
                    .to_string(),
            );
        }
    } else {
        fs::write(&ledger_path, normalized).map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn is_registry_asset_path(path: &str) -> bool {
    let allowed_roots = [
        "evidence/authoring/examples/",
        "evidence/authoring/patterns/",
        "evidence/authoring/negative/",
        "evidence/battle/workflows/",
        "evidence/compat/graph_schema/",
        "evidence/compat/export_bundle/",
        "evidence/compat/run_dir/",
        "evidence/compat/scenarios/",
        "evidence/cache/corrupt/",
        "evidence/cache/scenarios/",
        "evidence/cache/replay/",
        "evidence/fault/classes/",
        "evidence/fault/corrupt_runs/",
        "evidence/perf/scenarios/",
        "evidence/compare/scenarios/",
        "evidence/operator/scenarios/",
    ];
    if !allowed_roots.iter().any(|prefix| path.starts_with(prefix)) {
        return false;
    }
    path.ends_with(".json") || path.ends_with(".dag.json")
}

#[cfg(test)]
mod tests {
    use super::is_registry_asset_path;

    #[test]
    fn registry_asset_path_classifier_accepts_governed_roots_only() {
        assert!(is_registry_asset_path(
            "evidence/authoring/examples/example.dag.json"
        ));
        assert!(is_registry_asset_path("evidence/perf/scenarios/latency.json"));
        assert!(!is_registry_asset_path(
            "evidence/ownership/evidence_ledger.json"
        ));
        assert!(!is_registry_asset_path("evidence/perf/scenarios/readme.md"));
        assert!(!is_registry_asset_path("docs/reports/foundation/anything.json"));
    }
}

fn build_evidence_registry(root: &Path) -> Result<Value, String> {
    let ledger: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let entries = ledger["entries"]
        .as_array()
        .ok_or_else(|| "evidence ledger entries must be an array".to_string())?;
    let mut assets = Vec::new();
    for entry in entries {
        let path = entry
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "ledger entry path must be string".to_string())?;
        if !is_registry_asset_path(path) {
            continue;
        }
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("entry missing id for path {path}"))?;
        let kind = entry
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("entry missing kind for path {path}"))?;
        let owner = entry
            .get("owner")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("entry missing owner for path {path}"))?;
        let status = entry
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("entry missing status for path {path}"))?;
        let canonical_path = entry
            .get("canonical_path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("entry missing canonical_path for path {path}"))?;
        let consumers = entry
            .get("consumers")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("entry missing consumers for path {path}"))?;
        let trust_properties = entry
            .get("trust_properties")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let release_blocking = entry
            .get("release_blocking")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let duplicate_of = entry.get("duplicate_of").cloned().unwrap_or(Value::Null);
        let derived_from = entry.get("derived_from").cloned().unwrap_or(Value::Null);
        let simulation_status = entry
            .get("simulation_status")
            .and_then(Value::as_str)
            .unwrap_or("implemented");

        assets.push(json!({
            "registry_key": format!("{kind}:{id}"),
            "id": id,
            "kind": kind,
            "owner": owner,
            "status": status,
            "canonical_path": canonical_path,
            "consumers": consumers,
            "trust_properties": trust_properties,
            "release_blocking": release_blocking,
            "duplicate_of": duplicate_of,
            "derived_from": derived_from,
            "simulation_status": simulation_status
        }));
    }

    assets.sort_by(|a, b| {
        a["registry_key"]
            .as_str()
            .unwrap_or("")
            .cmp(b["registry_key"].as_str().unwrap_or(""))
    });

    Ok(json!({
        "version": "1",
        "owner": "bijux-dev-dag",
        "source": "evidence/ownership/evidence_ledger.json",
        "asset_count": assets.len(),
        "assets": assets
    }))
}

pub(super) fn run_evidence_registry_rebuild(out: &Path, check: bool) -> Result<(), String> {
    let root = repo_root()?;
    let registry = build_evidence_registry(&root)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(root.join(parent)).map_err(|err| err.to_string())?;
    }
    let payload = serde_json::to_string_pretty(&registry).map_err(|err| err.to_string())?;
    let out_path = root.join(out);
    if check {
        let current = fs::read_to_string(&out_path).map_err(|err| err.to_string())?;
        if current != format!("{payload}\n") && current != payload {
            return Err(format!(
                "evidence registry drift detected; run `bijux-dev-dag repo evidence-registry-rebuild` to refresh {}",
                out.display()
            ));
        }
    } else {
        fs::write(&out_path, format!("{payload}\n")).map_err(|err| err.to_string())?;
    }
    Ok(())
}

pub(super) fn run_evidence_registry_diff() -> Result<(), String> {
    run_evidence_registry_rebuild(
        Path::new("evidence/_meta/registries/evidence_registry.json"),
        true,
    )
}

fn collect_registry_asset_paths(root: &Path) -> Result<BTreeSet<String>, String> {
    let registry_path = root.join("evidence/_meta/registries/evidence_registry.json");
    let registry: Value =
        serde_json::from_str(&fs::read_to_string(registry_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let assets = registry["assets"]
        .as_array()
        .ok_or_else(|| "evidence registry assets must be an array".to_string())?;
    let mut paths = BTreeSet::new();
    for asset in assets {
        let canonical = asset["canonical_path"]
            .as_str()
            .ok_or_else(|| "registry canonical_path must be string".to_string())?;
        paths.insert(canonical.to_string());
    }
    Ok(paths)
}

pub(super) fn run_evidence_registry_orphans() -> Result<(), String> {
    let root = repo_root()?;
    let registry_paths = collect_registry_asset_paths(&root)?;
    let mut files = Vec::new();
    collect_all_files(&root.join("evidence"), &mut files)?;
    let mut orphans = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if is_registry_asset_path(&rel) && !registry_paths.contains(&rel) {
            orphans.push(rel);
        }
    }
    if orphans.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "evidence registry orphans detected: {}",
            orphans.join(", ")
        ))
    }
}

pub(super) fn run_evidence_registry_missing() -> Result<(), String> {
    let root = repo_root()?;
    let registry_paths = collect_registry_asset_paths(&root)?;
    let mut missing = Vec::new();
    for path in registry_paths {
        if !root.join(&path).exists() {
            missing.push(path);
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "evidence registry entries missing files: {}",
            missing.join(", ")
        ))
    }
}

pub(super) fn run_evidence_registry_verify() -> Result<(), String> {
    let root = repo_root()?;
    run_evidence_ledger_normalize(true)?;
    run_evidence_registry_diff()?;
    run_evidence_registry_orphans()?;
    run_evidence_registry_missing()?;

    let registry_path = root.join("evidence/_meta/registries/evidence_registry.json");
    let registry: Value =
        serde_json::from_str(&fs::read_to_string(registry_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let assets = registry["assets"]
        .as_array()
        .ok_or_else(|| "evidence registry assets must be an array".to_string())?;
    let mut keys = BTreeSet::new();
    let mut canonical = BTreeSet::new();
    for asset in assets {
        let key = asset["registry_key"]
            .as_str()
            .ok_or_else(|| "registry asset missing registry_key".to_string())?;
        let id = asset["id"]
            .as_str()
            .ok_or_else(|| "registry asset missing id".to_string())?;
        let path = asset["canonical_path"]
            .as_str()
            .ok_or_else(|| "registry asset missing canonical_path".to_string())?;
        if !keys.insert(key.to_string()) {
            return Err(format!("duplicate registry key detected: {key}"));
        }
        if !canonical.insert(path.to_string()) {
            return Err(format!("duplicate canonical identity detected: {path}"));
        }
        let duplicate_of = &asset["duplicate_of"];
        if let Some(reference) = duplicate_of.as_str() {
            if !reference.trim().is_empty() {
                let target_key =
                    key.split(':').next().unwrap_or("asset").to_string() + ":" + reference;
                let exists_by_key = assets.iter().any(|candidate| {
                    candidate["registry_key"].as_str() == Some(target_key.as_str())
                        || candidate["id"].as_str() == Some(reference)
                        || candidate["canonical_path"].as_str() == Some(reference)
                });
                if !exists_by_key {
                    return Err(format!(
                        "registry duplicate_of reference for `{id}` does not resolve: {reference}"
                    ));
                }
            }
        }
        let derived_from = &asset["derived_from"];
        if let Some(reference) = derived_from.as_str() {
            if !reference.trim().is_empty() {
                let exists_by_key = assets.iter().any(|candidate| {
                    candidate["registry_key"].as_str() == Some(reference)
                        || candidate["id"].as_str() == Some(reference)
                        || candidate["canonical_path"].as_str() == Some(reference)
                });
                if !exists_by_key {
                    return Err(format!(
                        "registry derived_from reference for `{id}` does not resolve: {reference}"
                    ));
                }
            }
        }
    }
    Ok(())
}
