use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn write_file(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, body).map_err(|e| e.to_string())
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

fn main() -> Result<(), String> {
    let root = repo_root();
    let policy_path = root.join("configs/dag/policy/human_output_governance.json");
    let raw = fs::read_to_string(policy_path).map_err(|e| e.to_string())?;
    let policy: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    let out_dir = root.join("docs/reports/foundation");
    let ref_path = root.join("docs/reference/OPERATOR_UX_REFERENCE_GENERATED.md");
    let example_root = root.join("evidence/operator/examples/human_output");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&example_root).map_err(|e| e.to_string())?;

    let families = policy["families"]
        .as_array()
        .ok_or_else(|| "families must be array".to_string())?;

    for fam in families {
        let family = fam["family"].as_str().unwrap_or("unknown");
        let family_dir = example_root.join(family);
        fs::create_dir_all(&family_dir).map_err(|e| e.to_string())?;

        let files = fam["snapshot_files"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let first = files.first().and_then(|v| v.as_str()).unwrap_or("");
        let second = files.get(1).and_then(|v| v.as_str()).unwrap_or(first);

        if !first.is_empty() {
            let src = root.join(first);
            let body = fs::read_to_string(src).map_err(|e| e.to_string())?;
            fs::write(family_dir.join("concise.txt"), body).map_err(|e| e.to_string())?;
        }
        if !second.is_empty() {
            let src = root.join(second);
            let body = fs::read_to_string(src).map_err(|e| e.to_string())?;
            fs::write(family_dir.join("detailed.txt"), body).map_err(|e| e.to_string())?;
        }
    }

    let mut inv = String::from("# Human Output Snapshot Inventory Report\n\n| Family | Snapshot tests | Snapshot files |\n| --- | --- | --- |\n");
    for fam in families {
        let family = fam["family"].as_str().unwrap_or("");
        let tests = fam["snapshot_tests"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        let files = fam["snapshot_files"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        inv.push_str(&format!("| `{family}` | `{tests}` | `{files}` |\n"));
    }
    write_file(
        &out_dir.join("human_output_snapshot_inventory_report.md"),
        &inv,
    )?;

    let mut rs_files = Vec::new();
    collect_rs_files(&root.join("crates/bijux-dag-app/tests"), &mut rs_files);
    collect_rs_files(&root.join("crates/bijux-core-dev/tests"), &mut rs_files);
    let mut test_sources = String::new();
    for file in rs_files {
        if let Ok(text) = fs::read_to_string(file) {
            test_sources.push_str(&text);
            test_sources.push('\n');
        }
    }

    let mut missing_rows: Vec<String> = Vec::new();
    for fam in families {
        let family = fam["family"].as_str().unwrap_or("");
        for t in fam["snapshot_tests"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            if let Some(test_name) = t.as_str() {
                if !test_sources.contains(test_name) {
                    missing_rows.push(format!(
                        "| `{family}` | `{test_name}` | `missing snapshot test` |"
                    ));
                }
            }
        }
        for s in fam["snapshot_files"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            if let Some(snap) = s.as_str() {
                if !root.join(snap).exists() {
                    missing_rows.push(format!(
                        "| `{family}` | `{snap}` | `missing snapshot file` |"
                    ));
                }
            }
        }
    }

    let missing_count = missing_rows.len();
    let mut missing = String::from("# Human Output Surfaces Without Snapshot Report\n\n| Family | Surface | Gap |\n| --- | --- | --- |\n");
    for row in &missing_rows {
        missing.push_str(row);
        missing.push('\n');
    }
    missing.push_str(&format!(
        "\nMissing human snapshot surfaces: {missing_count}\n"
    ));
    write_file(
        &out_dir.join("human_output_surfaces_without_snapshot_report.md"),
        &missing,
    )?;
    write_file(
        &out_dir.join("human_output_without_snapshot_tests_report.md"),
        &missing,
    )?;

    let mut concise_detail = String::from("# Concise vs Detailed Human Output Coverage Report\n\n| Family | Concise example | Detailed example | Distinct |\n| --- | --- | --- | --- |\n");
    for fam in families {
        let family = fam["family"].as_str().unwrap_or("");
        let c = example_root.join(family).join("concise.txt");
        let d = example_root.join(family).join("detailed.txt");
        let distinct = match (fs::read(&c), fs::read(&d)) {
            (Ok(cb), Ok(db)) => cb != db,
            _ => false,
        };
        concise_detail.push_str(&format!(
            "| `{family}` | `{}` | `{}` | `{}` |\n",
            c.strip_prefix(&root).unwrap_or(&c).to_string_lossy(),
            d.strip_prefix(&root).unwrap_or(&d).to_string_lossy(),
            distinct
        ));
    }
    write_file(
        &out_dir.join("concise_detailed_human_output_coverage_report.md"),
        &concise_detail,
    )?;

    let concise =
        fs::read(root.join("crates/bijux-dag-app/tests/snapshots/route_concise_wording.txt"))
            .unwrap_or_default();
    let detailed =
        fs::read(root.join("crates/bijux-dag-app/tests/snapshots/route_detailed_wording.txt"))
            .unwrap_or_default();
    let prove =
        fs::read(root.join("crates/bijux-dag-app/tests/snapshots/prove_human_output_contract.txt"))
            .unwrap_or_default();
    let verify = fs::read(
        root.join("crates/bijux-dag-app/tests/snapshots/verify_human_output_contract.txt"),
    )
    .unwrap_or_default();
    let mut drift = String::from(
        "# Wording Drift Equivalent Commands Report\n\n| Comparison | Result |\n| --- | --- |\n",
    );
    drift.push_str(&format!(
        "| route concise vs route detailed | `{}` |\n",
        if concise == detailed {
            "identical"
        } else {
            "different-as-expected"
        }
    ));
    drift.push_str(&format!(
        "| prove vs verify human output | `{}` |\n",
        if prove == verify {
            "identical"
        } else {
            "different-as-expected"
        }
    ));
    write_file(
        &out_dir.join("wording_drift_equivalent_commands_report.md"),
        &drift,
    )?;

    let mut ux = String::from("# Operator UX Reference (Generated)\n\nGenerated from human-output snapshots and governed examples.\n\n");
    for fam in families {
        let family = fam["family"].as_str().unwrap_or("unknown");
        ux.push_str(&format!("## {family}\n\n- Snapshot tests:\n"));
        for t in fam["snapshot_tests"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            if let Some(name) = t.as_str() {
                ux.push_str(&format!("  - `{name}`\n"));
            }
        }
        ux.push_str("- Examples:\n");
        ux.push_str(&format!(
            "  - `evidence/operator/examples/human_output/{family}/concise.txt`\n"
        ));
        ux.push_str(&format!(
            "  - `evidence/operator/examples/human_output/{family}/detailed.txt`\n\n"
        ));
    }
    write_file(&ref_path, &ux)?;

    Ok(())
}
