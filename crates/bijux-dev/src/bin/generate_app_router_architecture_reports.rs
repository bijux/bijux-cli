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
use std::fs;
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
    let app_src = root.join("crates/bijux-dag-app/src");
    let routes_dir = app_src.join("routes");
    write_report(
        &root.join("docs/reports/foundation/app_route_responsibilities_report.md"),
        responsibilities_report(&root)?,
    )?;
    write_report(
        &root.join("docs/reports/foundation/APP_ROUTE_BUSINESS_LOGIC_RESIDUE_REPORT.md"),
        business_logic_residue_report(&routes_dir)?,
    )?;
    write_report(
        &root.join("docs/reports/foundation/APP_ROUTE_COMPLEXITY_SCORE_REPORT.md"),
        complexity_report(&routes_dir)?,
    )?;
    write_report(
        &root.join("docs/reports/foundation/APP_MODULE_DEPENDENCY_GRAPH_REPORT.md"),
        dependency_graph_report(&app_src, &routes_dir)?,
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

fn write_report(path: &Path, body: String) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, body).map_err(|e| e.to_string())
}

fn responsibilities_report(root: &Path) -> Result<String, String> {
    let lib = fs::read_to_string(root.join("crates/bijux-dag-app/src/lib.rs"))
        .map_err(|e| e.to_string())?;
    let mut lines = vec![
        "# App Route Responsibilities Report".to_string(),
        "".to_string(),
        "Delegated command families from `lib.rs` to route modules:".to_string(),
        "".to_string(),
    ];
    for family in [
        "validate_routes",
        "plan_routes",
        "run_routes",
        "inspect_routes",
        "replay_routes",
        "diff_routes",
        "prove_verify_routes",
        "artifact_routes",
        "runs_routes",
        "surface_routes",
        "diagnostics_routes",
        "export_import_routes",
    ] {
        let delegated = lib.contains(&format!("routes::{family}::"));
        lines.push(format!("- `{family}`: {}", if delegated { "delegated" } else { "missing" }));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn business_logic_residue_report(routes_dir: &Path) -> Result<String, String> {
    let heavy_tokens = ["Runtime::new()", "build_plan(", "verify_run(", "inspect_artifact("];
    let mut lines = vec![
        "# App Route Business Logic Residue Report".to_string(),
        "".to_string(),
        "| file | residue_tokens |".to_string(),
        "| --- | --- |".to_string(),
    ];
    for entry in fs::read_dir(routes_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut count = 0usize;
        for token in heavy_tokens {
            count += content.matches(token).count();
        }
        lines.push(format!(
            "| `{}` | {} |",
            path.file_name().and_then(|v| v.to_str()).unwrap_or("unknown"),
            count
        ));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn complexity_report(routes_dir: &Path) -> Result<String, String> {
    let mut lines = vec![
        "# App Route Complexity Score Report".to_string(),
        "".to_string(),
        "| file | lines | branch_tokens | complexity_score |".to_string(),
        "| --- | --- | --- | --- |".to_string(),
    ];
    for entry in fs::read_dir(routes_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let line_count = content.lines().count();
        let branch_tokens = [" if ", " match ", " for ", " while ", "&&", "||"];
        let mut branches = 0usize;
        for token in branch_tokens {
            branches += content.matches(token).count();
        }
        let score = line_count + branches * 8;
        lines.push(format!(
            "| `{}` | {} | {} | {} |",
            path.file_name().and_then(|v| v.to_str()).unwrap_or("unknown"),
            line_count,
            branches,
            score
        ));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn dependency_graph_report(app_src: &Path, routes_dir: &Path) -> Result<String, String> {
    let mut lines = vec![
        "# App Module Dependency Graph Report".to_string(),
        "".to_string(),
        "```text".to_string(),
    ];
    let lib = fs::read_to_string(app_src.join("lib.rs")).map_err(|e| e.to_string())?;
    for module in [
        "validate_routes",
        "plan_routes",
        "run_routes",
        "inspect_routes",
        "replay_routes",
        "diff_routes",
        "prove_verify_routes",
        "artifact_routes",
        "runs_routes",
        "surface_routes",
        "diagnostics_routes",
        "export_import_routes",
    ] {
        if lib.contains(&format!("routes::{module}::")) {
            lines.push(format!("lib.rs -> routes/{module}.rs"));
        }
    }
    for entry in fs::read_dir(routes_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("rs") {
            continue;
        }
        let file = path.file_name().and_then(|v| v.to_str()).unwrap_or("unknown").to_string();
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("use crate::") {
                lines.push(format!("{file} -> {}", rest.trim_end_matches(';')));
            }
        }
    }
    lines.push("```".to_string());
    Ok(format!("{}\n", lines.join("\n")))
}
