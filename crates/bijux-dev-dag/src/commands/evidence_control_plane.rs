use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::evidence_access::load_registry_assets;
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
    if blocking_assets.is_empty() {
        return Err("release evidence set must include blocking_assets".to_string());
    }

    let registry_assets = load_registry_assets(&root)?;
    let registry_ids: BTreeSet<String> = registry_assets.iter().map(|a| a.id.clone()).collect();
    for collection in [blocking_assets, advisory_assets] {
        for asset in collection {
            let id = asset
                .as_str()
                .ok_or_else(|| "release evidence asset id must be a string".to_string())?;
            if !registry_ids.contains(id) {
                return Err(format!(
                    "release evidence set references unknown registry asset `{id}`"
                ));
            }
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
