use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceAsset {
    pub id: String,
    pub kind: String,
    pub canonical_path: String,
    pub consumers: Vec<String>,
    pub trust_properties: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryReleaseAsset {
    pub id: String,
    pub kind: String,
    pub owner: String,
    pub consumers: Vec<String>,
    pub trust_properties: Vec<String>,
    pub release_blocking: bool,
}

pub fn load_registry_assets(root: &Path) -> Result<Vec<EvidenceAsset>, String> {
    let payload = fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
        .map_err(|err| err.to_string())?;
    let registry: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    parse_registry_assets(&registry)
}

pub fn load_registry_release_blocking_flags(root: &Path) -> Result<BTreeMap<String, bool>, String> {
    let payload = fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
        .map_err(|err| err.to_string())?;
    let registry: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    let assets = registry["assets"]
        .as_array()
        .ok_or_else(|| "evidence registry assets must be an array".to_string())?;
    let mut flags = BTreeMap::new();
    for asset in assets {
        let id = asset["id"]
            .as_str()
            .ok_or_else(|| "registry asset missing string id".to_string())?
            .to_string();
        let release_blocking = asset["release_blocking"]
            .as_bool()
            .ok_or_else(|| format!("registry asset `{id}` missing release_blocking bool"))?;
        flags.insert(id, release_blocking);
    }
    Ok(flags)
}

pub fn load_registry_release_assets(
    root: &Path,
) -> Result<BTreeMap<String, RegistryReleaseAsset>, String> {
    let payload = fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
        .map_err(|err| err.to_string())?;
    let registry: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    let assets = registry["assets"]
        .as_array()
        .ok_or_else(|| "evidence registry assets must be an array".to_string())?;
    let mut map = BTreeMap::new();
    for asset in assets {
        let id = asset["id"]
            .as_str()
            .ok_or_else(|| "registry asset missing string id".to_string())?
            .to_string();
        let kind = asset["kind"]
            .as_str()
            .ok_or_else(|| format!("registry asset `{id}` missing kind"))?
            .to_string();
        let owner = asset["owner"]
            .as_str()
            .ok_or_else(|| format!("registry asset `{id}` missing owner"))?
            .to_string();
        let consumers = asset["consumers"]
            .as_array()
            .ok_or_else(|| format!("registry asset `{id}` has non-array consumers"))?
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .ok_or_else(|| format!("registry asset `{id}` has non-string consumer"))
                    .map(ToOwned::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let trust_properties = asset["trust_properties"]
            .as_array()
            .ok_or_else(|| format!("registry asset `{id}` has non-array trust_properties"))?
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .ok_or_else(|| {
                        format!("registry asset `{id}` has non-string trust property entry")
                    })
                    .map(ToOwned::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let release_blocking = asset["release_blocking"]
            .as_bool()
            .ok_or_else(|| format!("registry asset `{id}` missing release_blocking bool"))?;
        map.insert(
            id.clone(),
            RegistryReleaseAsset {
                id,
                kind,
                owner,
                consumers,
                trust_properties,
                release_blocking,
            },
        );
    }
    Ok(map)
}

pub fn parse_registry_assets(registry: &Value) -> Result<Vec<EvidenceAsset>, String> {
    let assets = registry["assets"]
        .as_array()
        .ok_or_else(|| "evidence registry assets must be an array".to_string())?;

    let mut parsed = Vec::new();
    let mut ids = BTreeSet::new();
    for asset in assets {
        let id = asset["id"]
            .as_str()
            .ok_or_else(|| "registry asset missing string id".to_string())?
            .to_string();
        let kind = asset["kind"]
            .as_str()
            .ok_or_else(|| format!("registry asset `{id}` missing kind"))?
            .to_string();
        let canonical_path = asset["canonical_path"]
            .as_str()
            .ok_or_else(|| format!("registry asset `{id}` missing canonical_path"))?
            .to_string();
        let consumers = asset["consumers"]
            .as_array()
            .ok_or_else(|| format!("registry asset `{id}` has non-array consumers"))?
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .ok_or_else(|| format!("registry asset `{id}` has non-string consumer"))
                    .map(ToOwned::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let trust_properties = asset["trust_properties"]
            .as_array()
            .ok_or_else(|| format!("registry asset `{id}` has non-array trust_properties"))?
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .ok_or_else(|| {
                        format!("registry asset `{id}` has non-string trust property entry")
                    })
                    .map(ToOwned::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !ids.insert(id.clone()) {
            return Err(format!("duplicate evidence asset id: `{id}`"));
        }
        parsed.push(EvidenceAsset {
            id,
            kind,
            canonical_path,
            consumers,
            trust_properties,
        });
    }
    parsed.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(parsed)
}

pub fn resolve_asset_by_id<'a>(
    assets: &'a [EvidenceAsset],
    asset_id: &str,
) -> Result<&'a EvidenceAsset, String> {
    assets
        .iter()
        .find(|asset| asset.id == asset_id)
        .ok_or_else(|| format!("evidence asset id not found: `{asset_id}`"))
}

pub fn resolve_assets_by_family<'a>(
    assets: &'a [EvidenceAsset],
    family: &str,
) -> Vec<&'a EvidenceAsset> {
    let mut selected: Vec<&EvidenceAsset> =
        assets.iter().filter(|asset| asset.kind == family).collect();
    selected.sort_by(|a, b| a.id.cmp(&b.id));
    selected
}

pub fn resolve_assets_by_trust_property<'a>(
    assets: &'a [EvidenceAsset],
    trust_property: &str,
) -> Vec<&'a EvidenceAsset> {
    let mut selected: Vec<&EvidenceAsset> = assets
        .iter()
        .filter(|asset| asset.trust_properties.iter().any(|tp| tp == trust_property))
        .collect();
    selected.sort_by(|a, b| a.id.cmp(&b.id));
    selected
}

pub fn resolve_assets_by_consumer<'a>(
    assets: &'a [EvidenceAsset],
    consumer: &str,
) -> Vec<&'a EvidenceAsset> {
    let mut selected: Vec<&EvidenceAsset> = assets
        .iter()
        .filter(|asset| asset.consumers.iter().any(|entry| entry == consumer))
        .collect();
    selected.sort_by(|a, b| a.id.cmp(&b.id));
    selected
}

pub fn classify_consumer_kind(consumer: &str) -> &'static str {
    if consumer.ends_with("-suite") {
        "suite"
    } else if consumer.ends_with("-contracts") {
        "contract"
    } else if consumer.contains("validate") {
        "command"
    } else if consumer.contains("runtime") {
        "runtime"
    } else if consumer.contains("performance") || consumer.contains("benchmark") {
        "performance"
    } else {
        "other"
    }
}

pub fn verify_registry_access_bypass(root: &Path) -> Result<(), String> {
    let source_root = root.join("crates");
    let mut files = Vec::new();
    collect_files(&source_root, &mut files)?;
    let allowlist = [
        "crates/bijux-core-dev/src/commands/evidence_access.rs",
        "crates/bijux-core-dev/src/commands/evidence_registry.rs",
        "crates/bijux-dag-testkit/src/lib.rs",
        "crates/bijux-core-dev/tests/evidence_registry_contracts.rs",
        "crates/bijux-core-dev/tests/evidence_family_boundary_contracts.rs",
        "crates/bijux-core-dev/tests/evidence_access_contracts.rs",
        "crates/bijux-core-dev/tests/evidence_control_plane_suites_contracts.rs",
        "crates/bijux-core-dev/tests/evidence_consumer_integrity_contracts.rs",
    ];

    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.ends_with(".rs") {
            continue;
        }
        if allowlist.iter().any(|entry| rel == *entry) {
            continue;
        }
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        let direct_read_patterns = [
            "read_to_string(root.join(\"evidence/_meta/registries/evidence_registry.json\"))",
            "read_to_string(&root.join(\"evidence/_meta/registries/evidence_registry.json\"))",
            "read_to_string(repo_root.join(\"evidence/_meta/registries/evidence_registry.json\"))",
            "read_to_string(&repo_root.join(\"evidence/_meta/registries/evidence_registry.json\"))",
        ];
        if direct_read_patterns
            .iter()
            .any(|pattern| content.contains(pattern))
        {
            violations.push(rel);
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "direct registry filesystem reads must use evidence_access helpers: {}",
            violations.join(", ")
        ))
    }
}

pub fn render_assets_to_consumers_report(assets: &[EvidenceAsset]) -> String {
    let mut lines = Vec::new();
    lines.push("# Evidence Assets To Consumers".to_string());
    lines.push(String::new());
    lines.push("Generated from `evidence/_meta/registries/evidence_registry.json`.".to_string());
    lines.push(String::new());
    lines.push("| Asset ID | Family | Consumers |".to_string());
    lines.push("| --- | --- | --- |".to_string());
    for asset in assets {
        let consumers = if asset.consumers.is_empty() {
            "-".to_string()
        } else {
            asset.consumers.join(", ")
        };
        lines.push(format!(
            "| `{}` | `{}` | `{}` |",
            asset.id, asset.kind, consumers
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub fn render_consumers_to_families_report(assets: &[EvidenceAsset]) -> String {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for asset in assets {
        for consumer in &asset.consumers {
            map.entry(consumer.clone())
                .or_default()
                .insert(asset.kind.clone());
        }
    }

    let mut lines = Vec::new();
    lines.push("# Evidence Consumers To Families".to_string());
    lines.push(String::new());
    lines.push("Generated from `evidence/_meta/registries/evidence_registry.json`.".to_string());
    lines.push(String::new());
    lines.push("| Consumer | Consumer Kind | Evidence Families |".to_string());
    lines.push("| --- | --- | --- |".to_string());
    for (consumer, families) in map {
        let families_list = families.into_iter().collect::<Vec<_>>().join(", ");
        lines.push(format!(
            "| `{}` | `{}` | `{}` |",
            consumer,
            classify_consumer_kind(&consumer),
            families_list
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

#[cfg(test)]
mod access_tests {
    use super::{
        classify_consumer_kind, render_assets_to_consumers_report,
        render_consumers_to_families_report, EvidenceAsset,
    };

    #[test]
    fn consumer_kind_classifier_is_stable() {
        assert_eq!(classify_consumer_kind("release-suite"), "suite");
        assert_eq!(classify_consumer_kind("cache-contracts"), "contract");
        assert_eq!(classify_consumer_kind("runtime-verify"), "runtime");
        assert_eq!(classify_consumer_kind("benchmark-latency"), "performance");
        assert_eq!(classify_consumer_kind("custom-consumer"), "other");
    }

    #[test]
    fn evidence_access_reports_render_expected_tables() {
        let assets = vec![EvidenceAsset {
            id: "cache-hit".to_string(),
            kind: "cache".to_string(),
            canonical_path: "evidence/cache/scenarios/cache-hit.json".to_string(),
            consumers: vec!["release-suite".to_string(), "runtime-verify".to_string()],
            trust_properties: vec!["deterministic".to_string()],
        }];

        let a2c = render_assets_to_consumers_report(&assets);
        assert!(a2c.contains("| `cache-hit` | `cache` | `release-suite, runtime-verify` |"));

        let c2f = render_consumers_to_families_report(&assets);
        assert!(c2f.contains("| `release-suite` | `suite` | `cache` |"));
        assert!(c2f.contains("| `runtime-verify` | `runtime` | `cache` |"));
    }
}

pub fn as_json(assets: &[&EvidenceAsset]) -> Value {
    json!(assets
        .iter()
        .map(|asset| {
            json!({
                "id": asset.id,
                "kind": asset.kind,
                "canonical_path": asset.canonical_path,
                "consumers": asset.consumers,
                "trust_properties": asset.trust_properties
            })
        })
        .collect::<Vec<_>>())
}

fn collect_files(root: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_ids_fail_loudly() {
        let payload = json!({
            "assets": [
                {"id":"a","kind":"authoring","canonical_path":"evidence/authoring/examples/a.json","consumers":[],"trust_properties":[]},
                {"id":"a","kind":"authoring","canonical_path":"evidence/authoring/examples/b.json","consumers":[],"trust_properties":[]}
            ]
        });
        let err = parse_registry_assets(&payload).expect_err("must reject duplicate ids");
        assert!(
            err.contains("duplicate evidence asset id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn deterministic_sort_by_asset_id() {
        let payload = json!({
            "assets": [
                {"id":"z","kind":"authoring","canonical_path":"evidence/authoring/examples/z.json","consumers":[],"trust_properties":[]},
                {"id":"a","kind":"authoring","canonical_path":"evidence/authoring/examples/a.json","consumers":[],"trust_properties":[]}
            ]
        });
        let first = parse_registry_assets(&payload).expect("parse");
        let second = parse_registry_assets(&payload).expect("parse");
        assert_eq!(first, second, "asset parsing must be deterministic");
        assert_eq!(first[0].id, "a");
        assert_eq!(first[1].id, "z");
    }

    #[test]
    fn missing_asset_lookup_has_helpful_error() {
        let payload = json!({
            "assets": [
                {"id":"x","kind":"authoring","canonical_path":"evidence/authoring/examples/x.json","consumers":[],"trust_properties":[]}
            ]
        });
        let assets = parse_registry_assets(&payload).expect("parse");
        let err = resolve_asset_by_id(&assets, "missing").expect_err("must fail");
        assert!(
            err.contains("evidence asset id not found: `missing`"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn typed_resolvers_select_expected_assets() {
        let assets = vec![
            EvidenceAsset {
                id: "a1".to_string(),
                kind: "authoring".to_string(),
                canonical_path: "evidence/authoring/examples/a1.json".to_string(),
                consumers: vec!["release-suite".to_string()],
                trust_properties: vec!["deterministic".to_string()],
            },
            EvidenceAsset {
                id: "c1".to_string(),
                kind: "cache".to_string(),
                canonical_path: "evidence/cache/scenarios/c1.json".to_string(),
                consumers: vec!["runtime-verify".to_string()],
                trust_properties: vec!["deterministic".to_string(), "traceable".to_string()],
            },
        ];

        let by_family = resolve_assets_by_family(&assets, "cache");
        assert_eq!(by_family.len(), 1);
        assert_eq!(by_family[0].id, "c1");

        let by_trust = resolve_assets_by_trust_property(&assets, "deterministic");
        assert_eq!(by_trust.len(), 2);

        let by_consumer = resolve_assets_by_consumer(&assets, "runtime-verify");
        assert_eq!(by_consumer.len(), 1);
        assert_eq!(by_consumer[0].id, "c1");
    }
}
