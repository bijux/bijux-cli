use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime::{backend_registry, execution_mode_report};
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde::Deserialize;
use sha2 as _;
use std::path::{Path, PathBuf};
use tempfile as _;

#[derive(Debug, Deserialize)]
struct SurfaceCatalog {
    surfaces: Vec<SurfaceEntry>,
}

#[derive(Debug, Deserialize)]
struct SurfaceEntry {
    module: String,
    category: String,
    owner: String,
    contract_test: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let root = repo_root()?;
    let catalog = load_catalog(&root)?;
    write_runtime_adapter_surface_inventory(&root, &catalog)?;
    write_backend_capability_report(&root)?;
    write_backend_support_matrix(&root)?;
    write_unsupported_capability_approximations(&root)?;
    write_backend_mode_lists(&root)?;
    Ok(())
}

fn repo_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "repo root not found".to_string())
}

fn load_catalog(root: &Path) -> Result<SurfaceCatalog, String> {
    let raw = std::fs::read_to_string(
        root.join("configs/dag/policy/runtime_adapter_surface_catalog.json"),
    )
    .map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

fn write_runtime_adapter_surface_inventory(
    root: &Path,
    catalog: &SurfaceCatalog,
) -> Result<(), String> {
    let mut lines = vec![
        "# Runtime Adapter Surface Inventory".to_string(),
        "".to_string(),
        "| module | category | owner | contract test |".to_string(),
        "| --- | --- | --- | --- |".to_string(),
    ];
    for entry in &catalog.surfaces {
        lines.push(format!(
            "| `{}` | {} | {} | `{}` |",
            entry.module, entry.category, entry.owner, entry.contract_test
        ));
    }
    write_report(
        root,
        "docs/reports/foundation/runtime_adapter_surface_inventory.md",
        &lines.join("\n"),
    )
}

fn write_backend_capability_report(root: &Path) -> Result<(), String> {
    let mut lines = vec![
        "# Backend Capability Matrix".to_string(),
        "".to_string(),
        "| backend | kind | env shaping | timeout | stream capture |".to_string(),
        "| --- | --- | --- | --- | --- |".to_string(),
    ];
    for row in backend_registry() {
        lines.push(format!(
            "| {} | {:?} | {} | {} | {} |",
            row.backend_name,
            row.kind,
            row.supports_env_shaping,
            row.supports_timeout,
            row.supports_stream_capture
        ));
    }
    write_report(root, "docs/reports/foundation/backend_capability_matrix.md", &lines.join("\n"))
}

fn write_backend_support_matrix(root: &Path) -> Result<(), String> {
    let mode = execution_mode_report();
    let mut lines = vec![
        "# Backend Support Matrix".to_string(),
        "".to_string(),
        "| surface | status |".to_string(),
        "| --- | --- |".to_string(),
    ];
    for item in mode.implemented {
        lines.push(format!("| {} | implemented |", item));
    }
    for item in mode.simulated {
        lines.push(format!("| {} | simulated |", item));
    }
    for item in mode.aspirational {
        lines.push(format!("| {} | aspirational |", item));
    }
    write_report(root, "docs/reports/foundation/backend_support_matrix.md", &lines.join("\n"))
}

fn write_unsupported_capability_approximations(root: &Path) -> Result<(), String> {
    let mode = execution_mode_report();
    let mut lines = vec![
        "# Unsupported Capability Approximations Report".to_string(),
        "".to_string(),
        "No unsupported capability is marked implemented. Unsupported surfaces remain simulated or aspirational.".to_string(),
        "".to_string(),
        "## Simulated".to_string(),
    ];
    for item in mode.simulated {
        lines.push(format!("- {}", item));
    }
    lines.push("".to_string());
    lines.push("## Aspirational".to_string());
    for item in mode.aspirational {
        lines.push(format!("- {}", item));
    }
    write_report(
        root,
        "docs/reports/foundation/unsupported_capability_approximations_report.md",
        &lines.join("\n"),
    )
}

fn write_backend_mode_lists(root: &Path) -> Result<(), String> {
    let mode = execution_mode_report();
    let implemented = format!(
        "# Implemented Backend Surfaces\n\n{}",
        mode.implemented
            .into_iter()
            .map(|entry| format!("- {}", entry))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let simulated = format!(
        "# Simulated Backend Surfaces\n\n{}",
        mode.simulated
            .into_iter()
            .map(|entry| format!("- {}", entry))
            .collect::<Vec<_>>()
            .join("\n")
    );
    write_report(
        root,
        "docs/reports/foundation/implemented_backend_surfaces_report.md",
        &implemented,
    )?;
    write_report(root, "docs/reports/foundation/simulated_backend_surfaces_report.md", &simulated)
}

fn write_report(root: &Path, rel: &str, content: &str) -> Result<(), String> {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    std::fs::write(path, format!("{content}\n")).map_err(|err| err.to_string())
}
