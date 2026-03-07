use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::evidence_access::load_registry_release_blocking_flags;
use super::repo_root;

pub(super) fn run_evidence_suite_policy_verify() -> Result<(), String> {
    let root = repo_root()?;
    let payload = fs::read_to_string(root.join("configs/policy/evidence_suite_policy.json"))
        .map_err(|err| err.to_string())?;
    let policy: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    let suites = policy["suites"]
        .as_array()
        .ok_or_else(|| "evidence suite policy must contain `suites` array".to_string())?;
    if suites.is_empty() {
        return Err("evidence suite policy must define at least one suite".to_string());
    }
    for suite in suites {
        let id = suite["id"]
            .as_str()
            .ok_or_else(|| "evidence suite policy entry missing id".to_string())?;
        let verify_command = suite["verify_command"]
            .as_str()
            .ok_or_else(|| format!("evidence suite policy entry `{id}` missing verify_command"))?;
        let mode = suite["mode"]
            .as_str()
            .ok_or_else(|| format!("evidence suite policy entry `{id}` missing mode"))?;
        if !["blocking", "advisory"].contains(&mode) {
            return Err(format!(
                "evidence suite policy entry `{id}` has invalid mode `{mode}`"
            ));
        }
        if !verify_command.starts_with("verify evidence-") {
            return Err(format!(
                "evidence suite policy entry `{id}` has invalid verify command `{verify_command}`"
            ));
        }
    }
    Ok(())
}

pub(super) fn run_evidence_release_set_verify() -> Result<(), String> {
    let root = repo_root()?;
    let payload = fs::read_to_string(root.join("evidence/release/release_evidence_set.json"))
        .map_err(|err| err.to_string())?;
    let release_set: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    let blocking_assets = release_set["blocking_assets"]
        .as_array()
        .ok_or_else(|| "release evidence set missing blocking_assets array".to_string())?;
    let advisory_assets = release_set["advisory_assets"]
        .as_array()
        .ok_or_else(|| "release evidence set missing advisory_assets array".to_string())?;
    let required_families = release_set["required_families"]
        .as_array()
        .ok_or_else(|| "release evidence set missing required_families array".to_string())?;
    let advisory_families = release_set["advisory_families"]
        .as_array()
        .ok_or_else(|| "release evidence set missing advisory_families array".to_string())?;
    let minimum_sets = release_set["minimum_blocking_sets"]
        .as_object()
        .ok_or_else(|| "release evidence set missing minimum_blocking_sets object".to_string())?;
    if blocking_assets.is_empty() {
        return Err("release evidence set must include blocking_assets".to_string());
    }

    let registry_release_flags = load_registry_release_blocking_flags(&root)?;
    let registry_payload =
        fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
            .map_err(|err| err.to_string())?;
    let registry: Value = serde_json::from_str(&registry_payload).map_err(|err| err.to_string())?;
    let registry_assets = registry["assets"]
        .as_array()
        .ok_or_else(|| "evidence registry assets must be an array".to_string())?;
    let mut registry_kind_by_id = BTreeMap::new();
    for asset in registry_assets {
        let id = asset["id"]
            .as_str()
            .ok_or_else(|| "registry asset missing id".to_string())?;
        let kind = asset["kind"]
            .as_str()
            .ok_or_else(|| format!("registry asset `{id}` missing kind"))?;
        registry_kind_by_id.insert(id.to_string(), kind.to_string());
    }

    if required_families.is_empty() {
        return Err("release evidence set required_families cannot be empty".to_string());
    }
    for family in required_families {
        let family = family
            .as_str()
            .ok_or_else(|| "required_families entries must be string".to_string())?;
        if family.trim().is_empty() {
            return Err("required_families cannot include empty values".to_string());
        }
    }
    for family in advisory_families {
        let family = family
            .as_str()
            .ok_or_else(|| "advisory_families entries must be string".to_string())?;
        if family.trim().is_empty() {
            return Err("advisory_families cannot include empty values".to_string());
        }
    }

    let mut blocking_ids = BTreeSet::new();
    let required_families_set: BTreeSet<String> = required_families
        .iter()
        .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
        .collect();
    let advisory_families_set: BTreeSet<String> = advisory_families
        .iter()
        .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
        .collect();
    let mut blocking_family_coverage = BTreeSet::new();
    for asset in blocking_assets {
        let id = asset
            .as_str()
            .ok_or_else(|| "release evidence asset id must be a string".to_string())?;
        if id.starts_with("examples/")
            || id.starts_with("benchmarks/")
            || id.starts_with("comparisons/")
        {
            return Err(format!(
                "release evidence set references legacy root path `{id}`"
            ));
        }
        if !id.starts_with("evidence/") {
            return Err(format!(
                "release evidence set must reference canonical evidence paths, got `{id}`"
            ));
        }
        let Some(is_release_blocking) = registry_release_flags.get(id) else {
            return Err(format!(
                "release evidence set references unknown registry asset `{id}`"
            ));
        };
        if !is_release_blocking {
            return Err(format!(
                "blocking release asset is not release_blocking in registry: `{id}`"
            ));
        }
        let kind = registry_kind_by_id
            .get(id)
            .ok_or_else(|| format!("registry kind missing for `{id}`"))?;
        if advisory_families_set.contains(kind) {
            return Err(format!(
                "ambiguous evidence classification: blocking asset `{id}` belongs to advisory family `{kind}`"
            ));
        }
        if !required_families_set.contains(kind) {
            return Err(format!(
                "ambiguous evidence classification: blocking asset `{id}` has family `{kind}` not listed in required_families"
            ));
        }
        blocking_family_coverage.insert(kind.clone());
        if !blocking_ids.insert(id.to_string()) {
            return Err(format!("duplicate blocking release asset id: `{id}`"));
        }
    }

    let mut advisory_ids = BTreeSet::new();
    for asset in advisory_assets {
        let id = asset
            .as_str()
            .ok_or_else(|| "release evidence asset id must be a string".to_string())?;
        let Some(is_release_blocking) = registry_release_flags.get(id) else {
            return Err(format!(
                "release evidence set references unknown registry asset `{id}`"
            ));
        };
        if *is_release_blocking {
            return Err(format!(
                "advisory release asset is marked release_blocking in registry: `{id}`"
            ));
        }
        let kind = registry_kind_by_id
            .get(id)
            .ok_or_else(|| format!("registry kind missing for `{id}`"))?;
        if required_families_set.contains(kind) {
            return Err(format!(
                "ambiguous evidence classification: advisory asset `{id}` belongs to required family `{kind}`"
            ));
        }
        if !advisory_families_set.contains(kind) {
            return Err(format!(
                "ambiguous evidence classification: advisory asset `{id}` has family `{kind}` not listed in advisory_families"
            ));
        }
        if blocking_ids.contains(id) {
            return Err(format!(
                "release evidence asset cannot be both blocking and advisory: `{id}`"
            ));
        }
        if !advisory_ids.insert(id.to_string()) {
            return Err(format!("duplicate advisory release asset id: `{id}`"));
        }
    }

    for required in ["replay", "cache", "operator"] {
        if !blocking_family_coverage.contains(required) {
            return Err(format!(
                "release evidence set missing blocking coverage for required trust family `{required}`"
            ));
        }
    }

    for (set_id, values) in minimum_sets {
        let minimum_assets = values
            .as_array()
            .ok_or_else(|| format!("minimum_blocking_sets `{set_id}` must be an array"))?;
        if minimum_assets.is_empty() {
            return Err(format!("minimum_blocking_sets `{set_id}` cannot be empty"));
        }
        for entry in minimum_assets {
            let id = entry
                .as_str()
                .ok_or_else(|| format!("minimum_blocking_sets `{set_id}` contains non-string"))?;
            if !blocking_ids.contains(id) {
                return Err(format!(
                    "minimum blocking set `{set_id}` references `{id}` not present in blocking_assets"
                ));
            }
        }
    }

    let expected_manifest = json!({
        "version": "1",
        "source": "evidence/release/release_evidence_set.json",
        "required_families": required_families_set,
        "advisory_families": advisory_families_set,
        "minimum_blocking_sets": minimum_sets,
        "blocking_assets": blocking_ids,
        "advisory_assets": advisory_ids
    });
    let manifest_path = root.join("evidence/release/release_evidence.json");
    if manifest_path.exists() {
        let manifest_payload = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
        let manifest: Value = serde_json::from_str(&manifest_payload).map_err(|err| err.to_string())?;
        if manifest != expected_manifest {
            return Err(
                "release evidence manifest drift detected; regenerate with `cargo run -p bijux-dev-dag -- repo release-evidence-report`"
                    .to_string(),
            );
        }
    }
    Ok(())
}

pub(super) fn run_evidence_summary_report(
    json_out: &Path,
    markdown_out: &Path,
) -> Result<(), String> {
    let root = repo_root()?;
    let policy_payload = fs::read_to_string(root.join("configs/policy/evidence_suite_policy.json"))
        .map_err(|err| err.to_string())?;
    let policy: Value = serde_json::from_str(&policy_payload).map_err(|err| err.to_string())?;
    let suites = policy["suites"]
        .as_array()
        .ok_or_else(|| "evidence suite policy must contain suites array".to_string())?;

    let mut blocking = Vec::new();
    let mut advisory = Vec::new();
    let mut markdown_lines = vec![
        "# Evidence Verification Summary".to_string(),
        String::new(),
        "This report lists governed evidence verify suites and their enforcement mode.".to_string(),
        String::new(),
        "| Suite ID | Verify Command | Mode |".to_string(),
        "| --- | --- | --- |".to_string(),
    ];

    for suite in suites {
        let id = suite["id"]
            .as_str()
            .ok_or_else(|| "suite id missing in evidence suite policy".to_string())?;
        let verify_command = suite["verify_command"]
            .as_str()
            .ok_or_else(|| format!("verify_command missing for `{id}`"))?;
        let mode = suite["mode"]
            .as_str()
            .ok_or_else(|| format!("mode missing for `{id}`"))?;
        markdown_lines.push(format!("| `{id}` | `{verify_command}` | `{mode}` |"));
        match mode {
            "blocking" => blocking.push(json!({ "id": id, "verify_command": verify_command })),
            "advisory" => advisory.push(json!({ "id": id, "verify_command": verify_command })),
            _ => return Err(format!("unsupported suite mode `{mode}` for `{id}`")),
        }
    }
    markdown_lines.push(String::new());

    let report = json!({
        "report_version": "1",
        "policy_source": "configs/policy/evidence_suite_policy.json",
        "blocking": blocking,
        "advisory": advisory,
    });
    fs::write(
        root.join(json_out),
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(root.join(markdown_out), markdown_lines.join("\n")).map_err(|err| err.to_string())?;
    println!(
        "{}",
        json!({
            "json_report": json_out.to_string_lossy(),
            "markdown_report": markdown_out.to_string_lossy(),
        })
    );
    Ok(())
}

pub(super) fn run_release_evidence_report(
    json_out: &Path,
    proves_out: &Path,
    limits_out: &Path,
    unsupported_out: &Path,
) -> Result<(), String> {
    let root = repo_root()?;
    let release_payload =
        fs::read_to_string(root.join("evidence/release/release_evidence_set.json"))
            .map_err(|err| err.to_string())?;
    let release_set: Value =
        serde_json::from_str(&release_payload).map_err(|err| err.to_string())?;
    let blocking_assets = release_set["blocking_assets"]
        .as_array()
        .ok_or_else(|| "release evidence set missing blocking_assets array".to_string())?;
    let advisory_assets = release_set["advisory_assets"]
        .as_array()
        .ok_or_else(|| "release evidence set missing advisory_assets array".to_string())?;
    let required_families = release_set["required_families"]
        .as_array()
        .ok_or_else(|| "release evidence set missing required_families array".to_string())?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or_else(|| "required_families entry must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let advisory_families = release_set["advisory_families"]
        .as_array()
        .ok_or_else(|| "release evidence set missing advisory_families array".to_string())?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or_else(|| "advisory_families entry must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let minimum_sets = release_set["minimum_blocking_sets"]
        .as_object()
        .ok_or_else(|| "release evidence set missing minimum_blocking_sets object".to_string())?;
    let mut minimum_map = BTreeMap::new();
    for (id, entries) in minimum_sets {
        let assets = entries
            .as_array()
            .ok_or_else(|| format!("minimum_blocking_sets `{id}` must be array"))?
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .ok_or_else(|| format!("minimum_blocking_sets `{id}` entry must be string"))
                    .map(ToOwned::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?;
        minimum_map.insert(id.clone(), assets);
    }

    let summary = json!({
        "version": "1",
        "source": "evidence/release/release_evidence_set.json",
        "required_families": required_families,
        "advisory_families": advisory_families,
        "minimum_blocking_sets": minimum_map,
        "blocking_assets": blocking_assets,
        "advisory_assets": advisory_assets
    });

    if let Some(parent) = root.join(json_out).parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(
        root.join(json_out),
        serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;

    let mut proves_lines = vec![
        "# What This Release Proves".to_string(),
        String::new(),
        "The following evidence assets are release-blocking and required for release readiness:"
            .to_string(),
        String::new(),
    ];
    for entry in blocking_assets {
        let id = entry
            .as_str()
            .ok_or_else(|| "blocking asset id must be string".to_string())?;
        proves_lines.push(format!("- `{id}`"));
    }
    proves_lines.push(String::new());
    proves_lines.push("Required release evidence families:".to_string());
    for family in &required_families {
        proves_lines.push(format!("- `{family}`"));
    }

    let mut limits_lines = vec![
        "# What This Release Does Not Prove".to_string(),
        String::new(),
        "The following evidence assets are advisory and are excluded from release-blocking readiness:".to_string(),
        String::new(),
    ];
    for entry in advisory_assets {
        let id = entry
            .as_str()
            .ok_or_else(|| "advisory asset id must be string".to_string())?;
        limits_lines.push(format!("- `{id}`"));
    }
    limits_lines.push(String::new());
    limits_lines.push("Advisory-only families:".to_string());
    for family in &advisory_families {
        limits_lines.push(format!("- `{family}`"));
    }

    fs::write(root.join(proves_out), proves_lines.join("\n")).map_err(|err| err.to_string())?;
    fs::write(root.join(limits_out), limits_lines.join("\n")).map_err(|err| err.to_string())?;
    let unsupported_lines = vec![
        "# Unsupported Or Simulated Areas".to_string(),
        String::new(),
        "This release does not claim production support for advisory-only evidence surfaces and simulated scenarios.".to_string(),
        String::new(),
        "Advisory evidence families:".to_string(),
        advisory_families
            .iter()
            .map(|family| format!("- `{family}`"))
            .collect::<Vec<_>>()
            .join("\n"),
    ];
    fs::write(root.join(unsupported_out), unsupported_lines.join("\n"))
        .map_err(|err| err.to_string())?;
    println!(
        "{}",
        json!({
            "json_report": json_out.to_string_lossy(),
            "proves_report": proves_out.to_string_lossy(),
            "limits_report": limits_out.to_string_lossy(),
            "unsupported_report": unsupported_out.to_string_lossy(),
        })
    );
    Ok(())
}
