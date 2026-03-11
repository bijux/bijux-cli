#![forbid(unsafe_code)]
//! Structural depth and workspace hygiene contracts.

use std::path::{Path, PathBuf};
use std::{fs};

fn collect_files_recursive(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn crate_src_path_depth_is_bounded() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");
    let crates_root = workspace_root.join("crates");

    let mut violations = Vec::<String>::new();
    for crate_entry in fs::read_dir(&crates_root).expect("missing crates directory").flatten() {
        let crate_path = crate_entry.path();
        if !crate_path.is_dir() {
            continue;
        }
        let src_path = crate_path.join("src");
        if !src_path.is_dir() {
            continue;
        }
        let mut files = Vec::<PathBuf>::new();
        collect_files_recursive(&src_path, &mut files);
        for file in files {
            let Ok(rel) = file.strip_prefix(&workspace_root) else {
                continue;
            };
            let depth = rel.components().count();
            if depth > 7 {
                violations.push(format!("depth={depth} path={}", rel.display()));
            }
        }
    }

    violations.sort();
    assert!(violations.is_empty(), "src depth violations:\n{}", violations.join("\n"));
}

#[test]
fn workspace_hygiene_forbids_legacy_root_directories_and_tokens() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");

    assert!(!workspace_root.join("scripts").exists());
    assert!(!workspace_root.join("target").exists());

    let mut rs_files = Vec::<PathBuf>::new();
    collect_rs_files(&crate_root.join("src"), &mut rs_files);
    collect_rs_files(&crate_root.join("tests"), &mut rs_files);

    let this_file = crate_root.join("tests/architecture/depth_limit.rs");
    for file in rs_files {
        if file == this_file {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        assert!(!text.contains("scripts"), "legacy token found in {}", file.display());
    }
}

#[test]
fn allowlists_are_centralized_under_configs_as_toml() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");

    let automation_allowlist = workspace_root.join("configs/allowlists/automation.toml");
    let public_api_allowlist = workspace_root.join("configs/allowlists/public_api.toml");

    assert!(automation_allowlist.exists());
    assert!(public_api_allowlist.exists());

    let automation_text = fs::read_to_string(&automation_allowlist).expect("read automation");
    let public_api_text = fs::read_to_string(&public_api_allowlist).expect("read public api");

    toml::from_str::<toml::Value>(&automation_text).expect("automation allowlist must be toml");
    toml::from_str::<toml::Value>(&public_api_text).expect("public api allowlist must be toml");

    assert!(!workspace_root.join(".github/maintenance_additions_allowlist.txt").exists());
    assert!(!workspace_root.join(".github/root_maintenance_additions_allowlist.txt").exists());
    assert!(!workspace_root.join(".github/public_api_allowlist.txt").exists());
}
