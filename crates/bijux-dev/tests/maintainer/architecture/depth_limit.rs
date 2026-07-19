#![forbid(unsafe_code)]
//! Structural depth and workspace hygiene contracts.

use std::fs;
use std::path::{Path, PathBuf};

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

fn collect_directories_named(root: &Path, target_name: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some(target_name) {
            out.push(path.clone());
        }
        collect_directories_named(&path, target_name, out);
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
            if depth > 8 {
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
    assert!(!workspace_root.join("packages").exists());
    assert!(!workspace_root.join("src").exists());
    assert!(!workspace_root.join("tests").exists());

    let mut rs_files = Vec::<PathBuf>::new();
    collect_rs_files(&crate_root.join("src"), &mut rs_files);
    collect_rs_files(&crate_root.join("tests"), &mut rs_files);

    let this_file = crate_root.join("tests/maintainer/architecture/depth_limit.rs");
    for file in rs_files {
        if file == this_file {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        for legacy_marker in ["join(\"scripts\")", "join(\"scripts/", "\"scripts/"] {
            assert!(!text.contains(legacy_marker), "legacy token found in {}", file.display());
        }
        for legacy_marker in ["join(\"packages\")", "join(\"packages/", "\"packages/"] {
            assert!(!text.contains(legacy_marker), "legacy token found in {}", file.display());
        }
    }
}

#[test]
fn legacy_exception_artifacts_are_absent() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");

    assert!(!workspace_root.join("configs/allowlists").exists());
    assert!(!workspace_root.join("configs/allowlists/automation.toml").exists());
    assert!(!workspace_root.join("configs/allowlists/public_api.toml").exists());
    assert!(!workspace_root.join(".github/maintenance_additions_allowlist.txt").exists());
    assert!(!workspace_root.join(".github/root_maintenance_additions_allowlist.txt").exists());
    assert!(!workspace_root.join(".github/public_api_allowlist.txt").exists());
}

#[test]
fn artifact_directories_exist_only_under_workspace_artifacts_root() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");
    let allowed_root = workspace_root.join("artifacts");

    let mut artifact_dirs = Vec::<PathBuf>::new();
    collect_directories_named(&workspace_root, "artifacts", &mut artifact_dirs);
    artifact_dirs.sort();

    let violations: Vec<String> = artifact_dirs
        .into_iter()
        .filter(|path| path != &allowed_root && !path.starts_with(&allowed_root))
        .filter(|path| {
            // Allow source modules named `artifacts` (for example
            // `crates/*/src/artifacts`) while still enforcing that generated
            // artifact directories remain under workspace `artifacts/`.
            let rel = path
                .strip_prefix(&workspace_root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            !rel.starts_with("crates/") || !rel.contains("/src/artifacts")
        })
        .filter_map(|path| {
            path.strip_prefix(&workspace_root).ok().map(|relative| relative.display().to_string())
        })
        .collect();

    assert!(
        violations.is_empty(),
        "artifact directories must live under workspace artifacts/: \n{}",
        violations.join("\n")
    );
}
