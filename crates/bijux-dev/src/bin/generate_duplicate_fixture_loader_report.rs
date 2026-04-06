use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("repo root")
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
}

fn extract_helper_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let starts = [
        "fn load_",
        "fn read_",
        "fn fixture_path",
        "fn fixture_dir",
        "fn fixtures_root",
        "fn parse_fixture",
    ];
    if !starts.iter().any(|p| trimmed.starts_with(p)) {
        return None;
    }
    let after_fn = trimmed.strip_prefix("fn ")?;
    let name: String =
        after_fn.chars().take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_').collect();
    if name.is_empty() {
        None
    } else {
        let valid_prefix = name.starts_with("load_")
            || name.starts_with("read_")
            || name == "fixture_path"
            || name == "fixture_dir"
            || name == "fixtures_root"
            || name == "parse_fixture";
        valid_prefix.then_some(name)
    }
}

fn main() -> Result<(), String> {
    let root = repo_root();
    let out = root.join("docs/reports/foundation/duplicate_fixture_loader_helpers_report.md");

    let scan_roots = [
        root.join("crates/bijux-dag-app"),
        root.join("crates/bijux-dag-runtime"),
        root.join("crates/bijux-dag-artifacts"),
        root.join("crates/bijux-dev"),
    ];

    let mut files = Vec::new();
    for dir in scan_roots {
        collect_rs_files(&dir, &mut files);
    }
    files.sort();

    let mut occurrences: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in files {
        let rel =
            file.strip_prefix(&root).map_err(|e| e.to_string())?.to_string_lossy().to_string();
        let content = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        for (idx, line) in content.lines().enumerate() {
            if let Some(name) = extract_helper_name(line) {
                occurrences.entry(name).or_default().push(format!("{rel}:{}", idx + 1));
            }
        }
    }

    let mut markdown = String::new();
    markdown.push_str("# Duplicate Fixture Loader Helpers Report\n\n");
    markdown.push_str(
        "Generated from fixture loader helper function signatures in app/runtime/artifacts/dev-dag crates.\n\n",
    );
    markdown.push_str("| Helper name | Occurrences | Locations |\n");
    markdown.push_str("| --- | --- | --- |\n");

    for (name, locs) in &occurrences {
        let joined = locs.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join("<br>");
        markdown.push_str(&format!("| `{name}` | {} | {joined} |\n", locs.len()));
    }

    let duplicate_count = occurrences.values().filter(|locs| locs.len() > 1).count();
    markdown.push_str("\n");
    markdown.push_str(&format!("Duplicate helper names: {duplicate_count}\n"));

    fs::write(out, markdown).map_err(|e| e.to_string())?;
    Ok(())
}
