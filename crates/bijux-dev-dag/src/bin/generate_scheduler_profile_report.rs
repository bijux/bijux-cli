#[cfg(test)]
use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime::scheduler_contract_profile;
use clap as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use std::path::{Path, PathBuf};
use tempfile as _;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let root = repo_root()?;
    let profile = scheduler_contract_profile();
    let fixture_sources = scheduler_fixture_sources(&root)?;
    if fixture_sources.is_empty() {
        return Err("no scheduler fixture sources discovered".to_string());
    }
    let report = json!({
        "format": "scheduler-profile/v0.2",
        "canonical_unit": enum_token(format!("{:?}", profile.canonical_unit)),
        "model": enum_token(format!("{:?}", profile.model)),
        "priority_model": enum_token(format!("{:?}", profile.priority_model)),
        "ready_tie_break": enum_token(format!("{:?}", profile.ready_tie_break)),
        "fixture_sources": fixture_sources,
    });
    let out = root.join("docs/reports/foundation/scheduler_profile_report.json");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    std::fs::write(
        &out,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
        ),
    )
    .map_err(|err| err.to_string())
}

fn repo_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "repo root not found".to_string())
}

fn scheduler_fixture_sources(root: &Path) -> Result<Vec<String>, String> {
    let tests_root = root.join("crates/bijux-dag-runtime/tests");
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&tests_root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.contains("scheduler") {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        entries.push(rel);
    }
    entries.sort();
    entries.dedup();
    Ok(entries)
}

fn enum_token(input: String) -> String {
    let mut out = String::new();
    for (idx, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() && idx > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}
