use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use serde as _;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn collect_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        out.push(path.to_path_buf());
        return;
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_files(&p, out);
            } else if p.is_file() {
                out.push(p);
            }
        }
    }
}

fn read_policy(root: &Path) -> Result<serde_json::Value, String> {
    let raw = fs::read_to_string(root.join("configs/policy/fixture_family_governance.json"))
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn family_files(root: &Path, family_obj: &serde_json::Value) -> Vec<String> {
    let mut files = Vec::new();
    let roots = family_obj["roots"].as_array().cloned().unwrap_or_default();
    for r in roots {
        if let Some(rel) = r.as_str() {
            let full = root.join(rel);
            let mut gathered = Vec::new();
            collect_files(&full, &mut gathered);
            for file in gathered {
                if let Ok(stripped) = file.strip_prefix(root) {
                    files.push(stripped.to_string_lossy().to_string());
                }
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

fn write_file(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, body).map_err(|e| e.to_string())
}

fn all_text_sources(root: &Path) -> Vec<PathBuf> {
    let roots = [
        "crates",
        "configs",
        "docs",
        "make",
        ".github",
        "Cargo.toml",
        "Makefile",
    ];
    let mut files = Vec::new();
    for rel in roots {
        let p = root.join(rel);
        if p.exists() {
            collect_files(&p, &mut files);
        }
    }
    files
}

fn main() -> Result<(), String> {
    let root = repo_root();
    let policy = read_policy(&root)?;
    let families = policy["governed_families"]
        .as_array()
        .ok_or_else(|| "governed_families must be array".to_string())?;

    let out_dir = root.join("docs/reports/foundation");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    let mut family_to_files: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for family_obj in families {
        let family = family_obj["family"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let owner_suite = family_obj["owner_suite"].as_str().unwrap_or("");
        let owner_crate = family_obj["owner_crate"].as_str().unwrap_or("");
        let purpose = family_obj["fixture_purpose"].as_str().unwrap_or("");

        let files = family_files(&root, family_obj);
        family_to_files.insert(family.clone(), files.clone());

        let mut body = String::new();
        body.push_str(&format!("# {} Fixture Inventory Report\n\n", {
            let mut c = family.chars();
            match c.next() {
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                None => family.clone(),
            }
        }));
        body.push_str(&format!("- Purpose: {purpose}\n"));
        body.push_str(&format!("- Owner suite: {owner_suite}\n"));
        body.push_str(&format!("- Owner crate: {owner_crate}\n\n"));
        body.push_str("| Fixture path | Owner suite | Owner crate |\n| --- | --- | --- |\n");
        for f in &files {
            body.push_str(&format!("| `{f}` | `{owner_suite}` | `{owner_crate}` |\n"));
        }
        body.push_str(&format!("\nTotal fixtures: {}\n", files.len()));
        write_file(
            &out_dir.join(format!("{family}_fixture_inventory_report.md")),
            &body,
        )?;
    }

    let mut missing_owner = String::from("# Fixture Governance Missing Owner Report\n\n| Family | Missing owner suite | Missing owner crate |\n| --- | --- | --- |\n");
    let mut missing_suite = String::from(
        "# Fixtures With No Owning Suite Report\n\n| Family | Owner suite |\n| --- | --- |\n",
    );
    let mut missing_crate = String::from(
        "# Fixtures With No Owning Crate Report\n\n| Family | Owner crate |\n| --- | --- |\n",
    );

    for family_obj in families {
        let family = family_obj["family"].as_str().unwrap_or("");
        let owner_suite = family_obj["owner_suite"].as_str().unwrap_or("");
        let owner_crate = family_obj["owner_crate"].as_str().unwrap_or("");
        let ms = owner_suite.is_empty();
        let mc = owner_crate.is_empty();
        missing_owner.push_str(&format!("| `{family}` | `{ms}` | `{mc}` |\n"));
        if ms {
            missing_suite.push_str(&format!("| `{family}` | `{owner_suite}` |\n"));
        }
        if mc {
            missing_crate.push_str(&format!("| `{family}` | `{owner_crate}` |\n"));
        }
    }
    write_file(
        &out_dir.join("fixture_governance_missing_owner_report.md"),
        &missing_owner,
    )?;
    write_file(
        &out_dir.join("fixtures_with_no_owning_suite_report.md"),
        &missing_suite,
    )?;
    write_file(
        &out_dir.join("fixtures_with_no_owning_crate_report.md"),
        &missing_crate,
    )?;

    let sources = all_text_sources(&root);
    let mut unreferenced = String::from(
        "# Unreferenced Fixtures Report\n\n| Family | Fixture path |\n| --- | --- |\n",
    );
    for (family, files) in &family_to_files {
        for file in files {
            let mut found = false;
            for src in &sources {
                if let Ok(text) = fs::read_to_string(src) {
                    if text.contains(file) {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                unreferenced.push_str(&format!("| `{family}` | `{file}` |\n"));
            }
        }
    }
    write_file(
        &out_dir.join("UNREFERENCED_FIXTURES_REPORT.md"),
        &unreferenced,
    )?;

    let mut hash_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for files in family_to_files.values() {
        for rel in files {
            let full = root.join(rel);
            if let Ok(bytes) = fs::read(&full) {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let hash = hex::encode(hasher.finalize());
                hash_map.entry(hash).or_default().insert(rel.clone());
            }
        }
    }
    let mut dup = String::from(
        "# Duplicate Fixtures Semantic Hash Report\n\n| SHA-256 | Fixture paths |\n| --- | --- |\n",
    );
    for (hash, paths) in &hash_map {
        if paths.len() > 1 {
            let joined = paths
                .iter()
                .map(|p| format!("`{p}`"))
                .collect::<Vec<_>>()
                .join("<br>");
            dup.push_str(&format!("| `{hash}` | {joined} |\n"));
        }
    }
    write_file(
        &out_dir.join("duplicate_fixtures_semantic_hash_report.md"),
        &dup,
    )?;

    let patterns = policy["legacy_schema_field_patterns"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut stale = String::from("# Stale Fixture Schema Field Report\n\n| Fixture path | Legacy field pattern |\n| --- | --- |\n");
    let mut stale_seen = BTreeSet::new();
    for p in patterns {
        if let Some(pattern) = p.as_str() {
            for files in family_to_files.values() {
                for rel in files {
                    let full = root.join(rel);
                    if let Ok(text) = fs::read_to_string(&full) {
                        if text.contains(pattern) {
                            let row = format!("| `{rel}` | `{pattern}` |\n");
                            if stale_seen.insert(row.clone()) {
                                stale.push_str(&row);
                            }
                        }
                    }
                }
            }
        }
    }
    write_file(
        &out_dir.join("stale_fixture_schema_field_report.md"),
        &stale,
    )?;

    let mut quick = String::from("# Fixture Governance Quick Reference\n\nPolicy source: `configs/policy/fixture_family_governance.json`\n\n## Governed families\n\n| Family | Purpose | Owner | Lane | Taxonomy |\n| --- | --- | --- | --- | --- |\n");
    for family_obj in families {
        quick.push_str(&format!(
            "| `{}` | {} | `{}` | `{}` | `{}` |\n",
            family_obj["family"].as_str().unwrap_or(""),
            family_obj["fixture_purpose"].as_str().unwrap_or(""),
            family_obj["fixture_owner"].as_str().unwrap_or(""),
            family_obj["fixture_lane"].as_str().unwrap_or(""),
            family_obj["fixture_taxonomy"].as_str().unwrap_or(""),
        ));
    }
    quick.push_str("\n## Generated reports\n\n");
    for family in family_to_files.keys() {
        quick.push_str(&format!(
            "- `docs/reports/foundation/{family}_fixture_inventory_report.md`\n"
        ));
    }
    for rel in [
        "fixture_governance_missing_owner_report.md",
        "fixtures_with_no_owning_suite_report.md",
        "fixtures_with_no_owning_crate_report.md",
        "UNREFERENCED_FIXTURES_REPORT.md",
        "duplicate_fixtures_semantic_hash_report.md",
        "stale_fixture_schema_field_report.md",
    ] {
        quick.push_str(&format!("- `docs/reports/foundation/{rel}`\n"));
    }
    write_file(
        &root.join("docs/reference/FIXTURE_GOVERNANCE_QUICK_REFERENCE.md"),
        &quick,
    )?;

    Ok(())
}
