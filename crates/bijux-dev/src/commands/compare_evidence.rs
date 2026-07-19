use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> Result<PathBuf, String> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

pub(super) fn run_compare_evidence_policy_verify() -> Result<(), String> {
    let root = repo_root()?;
    let metadata_path = root.join("evidence/compare/metadata.json");
    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let purpose = metadata["purpose"]
        .as_str()
        .ok_or_else(|| "comparison metadata missing purpose".to_string())?;
    if purpose.trim().is_empty() {
        return Err("comparison metadata purpose must be non-empty".to_string());
    }

    let scenarios = metadata["scenarios"]
        .as_object()
        .ok_or_else(|| "comparison metadata scenarios must be object".to_string())?;
    let mut factual_count = 0usize;
    for (scenario_path, entry) in scenarios {
        if !root.join(scenario_path).exists() {
            return Err(format!(
                "comparison metadata references missing scenario file: {scenario_path}"
            ));
        }
        let scenario_class = entry["scenario_class"]
            .as_str()
            .ok_or_else(|| format!("scenario_class missing for {scenario_path}"))?;
        if !["factual", "descriptive"].contains(&scenario_class) {
            return Err(format!("invalid scenario_class `{scenario_class}` for {scenario_path}"));
        }
        if scenario_class == "factual" {
            factual_count += 1;
        }
        let bijux_asset = entry["bijux_evidence_asset"]
            .as_str()
            .ok_or_else(|| format!("bijux_evidence_asset missing for {scenario_path}"))?;
        if !root.join(bijux_asset).exists() {
            return Err(format!(
                "comparison scenario `{scenario_path}` points to missing bijux evidence asset `{bijux_asset}`"
            ));
        }
        let limits = entry["non_equivalence_limits"]
            .as_array()
            .ok_or_else(|| format!("non_equivalence_limits missing for {scenario_path}"))?;
        if limits.is_empty() {
            return Err(format!(
                "comparison scenario `{scenario_path}` must declare non_equivalence_limits"
            ));
        }
        let release_blocking = entry["release_blocking"]
            .as_bool()
            .ok_or_else(|| format!("release_blocking missing or invalid for {scenario_path}"))?;
        let measured = entry["measured_bijux_side"]
            .as_bool()
            .ok_or_else(|| format!("measured_bijux_side missing for {scenario_path}"))?;
        if release_blocking && !measured {
            return Err(format!(
                "comparison scenario `{scenario_path}` cannot be release_blocking unless measured_bijux_side is true"
            ));
        }
    }
    if factual_count < 5 {
        return Err(format!(
            "comparison evidence is underspecified: expected at least 5 factual scenarios, found {factual_count}"
        ));
    }
    Ok(())
}

pub(super) fn run_comparison_harness_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/COMPARISON_HARNESS_CONTRACT.md",
        "docs/bijux-dag/interfaces/comparison-report-format.md",
        "docs/bijux-dag/quality/comparison-limitations.md",
        "docs/bijux-dag/quality/comparison-evidence-surfaces.md",
        "evidence/compare/baselines/bijux_v1.json",
        "evidence/compare/metadata.json",
        "crates/bijux-dag-app/tests/comparison_harness_contract.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "comparison harness required surfaces missing: {}",
            missing.join(", ")
        ));
    }

    run_compare_evidence_policy_verify()?;

    let scenario_dir = root.join("evidence/compare/scenarios");
    let mut scenario_count = 0usize;
    for entry in fs::read_dir(&scenario_dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|v| v.to_str()) == Some("json") {
            scenario_count += 1;
        }
    }
    if scenario_count < 5 {
        return Err(format!(
            "comparison harness requires at least 5 canonical scenarios, found {}",
            scenario_count
        ));
    }

    let mut violations = Vec::new();
    let docs_dir = root.join("docs");
    let mut stack = vec![docs_dir];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            for line in text.lines() {
                let lower = line.to_ascii_lowercase();
                let vague_superiority = lower.contains("superior")
                    || lower.contains("best dag")
                    || lower.contains("better than");
                if vague_superiority
                    && !line.contains("comparisons/")
                    && !line.contains("evidence/compare/")
                {
                    violations.push(format!("{rel}: {}", line.trim()));
                }
            }
        }
    }
    if !violations.is_empty() {
        return Err(format!(
            "vague superiority language without comparison evidence: {}",
            violations.join(" | ")
        ));
    }
    Ok(())
}

pub(super) fn run_comparison_evidence_report() -> Result<(), String> {
    let root = repo_root()?;
    run_compare_evidence_policy_verify()?;
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compare/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let scenarios = metadata["scenarios"]
        .as_object()
        .ok_or_else(|| "comparison metadata scenarios must be object".to_string())?;

    let mut factual = Vec::new();
    let mut descriptive = Vec::new();
    for (path, entry) in scenarios {
        let row = json!({
            "path": path,
            "target_system": entry["target_system"],
            "bijux_evidence_asset": entry["bijux_evidence_asset"],
            "non_equivalence_limits": entry["non_equivalence_limits"],
            "release_blocking": entry["release_blocking"]
        });
        if entry["scenario_class"].as_str() == Some("descriptive") {
            descriptive.push(row);
        } else {
            factual.push(row);
        }
    }

    let baseline: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compare/baselines/bijux_v1.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let baseline_count =
        baseline.get("scenarios").and_then(Value::as_array).map(|v| v.len()).unwrap_or(0);

    let payload = json!({
        "purpose": metadata["purpose"],
        "factual": factual,
        "interpretation_only": descriptive,
        "bijux_baseline_entries": baseline_count,
        "fact_vs_interpretation_report": "evidence/reports/comparison_fact_vs_interpretation.md"
    });
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::repo_root;

    #[test]
    fn repo_root_points_to_workspace_root() {
        let root = repo_root().expect("repo root");
        assert!(root.join("Cargo.toml").exists());
        assert!(root.join("crates").is_dir());
    }
}
