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
use std::path::{Path, PathBuf};
use tempfile as _;

const FAMILIES: &[&str] = &[
    "validate",
    "plan",
    "run",
    "inspect",
    "history",
    "replay",
    "diff",
    "prove",
    "export",
    "import",
    "artifact",
    "capabilities",
    "explain",
    "cache",
];

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let root = repo_root()?;
    let tests_dir = root.join("crates/bijux-dag-app/tests");
    let mut route_hits: BTreeMap<&'static str, usize> = FAMILIES.iter().map(|f| (*f, 0)).collect();
    let mut schema_hits: BTreeMap<&'static str, usize> = FAMILIES.iter().map(|f| (*f, 0)).collect();

    for entry in std::fs::read_dir(&tests_dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
        for family in FAMILIES {
            if raw.contains(family) {
                *route_hits.get_mut(family).expect("family hit") += 1;
            }
            if raw.contains("json") && raw.contains(family) {
                *schema_hits.get_mut(family).expect("schema hit") += 1;
            }
        }
    }

    write_markdown_table(
        &root.join("docs/reports/foundation/app_route_coverage_by_command_family.md"),
        "App Route Coverage By Command Family",
        &route_hits,
    )?;
    write_markdown_table(
        &root.join("docs/reports/foundation/app_response_schema_coverage_by_command_family.md"),
        "App Response Schema Coverage By Command Family",
        &schema_hits,
    )?;
    Ok(())
}

fn repo_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "repo root not found".to_string())
}

fn write_markdown_table(
    out_path: &Path,
    title: &str,
    counts: &BTreeMap<&'static str, usize>,
) -> Result<(), String> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let mut lines = vec![
        format!("# {title}"),
        "".to_string(),
        "| command_family | coverage_signal_count |".to_string(),
        "| --- | --- |".to_string(),
    ];
    for (family, count) in counts {
        lines.push(format!("| {} | {} |", family, count));
    }
    std::fs::write(out_path, format!("{}\n", lines.join("\n"))).map_err(|err| err.to_string())
}
