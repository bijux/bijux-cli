#![forbid(unsafe_code)]
//! Module layout contracts for durable dev-cli crate architecture.

use std::path::Path;
use std::{fs, path::PathBuf};

#[test]
fn command_and_status_contract_namespaces_exist() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(crate_root.join("src/commands").is_dir());
    assert!(crate_root.join("src/status_contracts").is_dir());
}

#[test]
fn legacy_native_directory_names_are_removed() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!crate_root
        .join("src/contracts/maintenance/native/catalog")
        .exists());
    assert!(!crate_root
        .join("src/contracts/maintenance/native/handlers")
        .exists());
}

#[test]
fn legacy_alias_modules_are_removed() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!crate_root.join("src/application").exists());
    assert!(!crate_root.join("src/features").exists());
}

#[test]
fn native_contract_modules_use_domain_first_filenames() {
    let native_mod = include_str!("../src/contracts/maintenance/native/mod.rs");
    assert!(native_mod.contains("mod executors;"));
    assert!(native_mod.contains("mod specs;"));

    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let native_dir = crate_root.join("src/contracts/maintenance/native");
    let mut violations = Vec::new();

    let Ok(entries) = fs::read_dir(&native_dir) else {
        panic!(
            "missing native contracts directory at {}",
            native_dir.display()
        );
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().map_or(true, |ext| ext != "rs") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name == "mod.rs" || file_name == "executors.rs" || file_name == "specs.rs" {
            continue;
        }
        if file_name.starts_with("executor_") || file_name.starts_with("spec_") {
            violations.push(format!(
                "{} uses legacy type-first prefix",
                path.strip_prefix(crate_root).unwrap_or(&path).display()
            ));
        }
        if !file_name.ends_with("_executor.rs") && !file_name.ends_with("_spec.rs") {
            violations.push(format!(
                "{} must end with _executor.rs or _spec.rs",
                path.strip_prefix(crate_root).unwrap_or(&path).display()
            ));
        }
    }

    violations.sort();
    assert!(
        violations.is_empty(),
        "native contract file naming violations:\n{}",
        violations.join("\n")
    );
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

#[test]
fn workspace_root_scripts_directory_is_removed_and_name_is_blocked() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");
    assert!(
        !workspace_root.join("scripts").exists(),
        "workspace root must not define scripts/ directory"
    );

    let mut rs_files = Vec::new();
    collect_rs_files(&crate_root.join("src"), &mut rs_files);
    collect_rs_files(&crate_root.join("tests"), &mut rs_files);

    let this_file = crate_root.join("tests/module_layout_contracts.rs");
    for file in rs_files {
        if file == this_file {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        assert!(
            !text.contains("scripts"),
            "forbidden legacy token `scripts` found in {}",
            file.display()
        );
    }
}

#[test]
fn workspace_crate_src_tree_depth_is_bounded() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");
    let crates_root = workspace_root.join("crates");

    let mut violations = Vec::<String>::new();
    let Ok(crate_entries) = fs::read_dir(&crates_root) else {
        panic!("missing crates directory at {}", crates_root.display());
    };
    for crate_entry in crate_entries.flatten() {
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
    assert!(
        violations.is_empty(),
        "crate src path depth must be <= 7 for every file under crates/*/src; violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn allowlists_are_centralized_under_config_as_toml() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");

    let automation_allowlist = workspace_root.join("config/allowlists/automation.toml");
    let public_api_allowlist = workspace_root.join("config/allowlists/public_api.toml");

    assert!(
        automation_allowlist.exists(),
        "missing {}",
        automation_allowlist.display()
    );
    assert!(
        public_api_allowlist.exists(),
        "missing {}",
        public_api_allowlist.display()
    );

    let automation_text = fs::read_to_string(&automation_allowlist).expect("read automation");
    let public_api_text = fs::read_to_string(&public_api_allowlist).expect("read public api");

    toml::from_str::<toml::Value>(&automation_text)
        .expect("automation allowlist must be valid toml");
    toml::from_str::<toml::Value>(&public_api_text)
        .expect("public api allowlist must be valid toml");

    assert!(!workspace_root
        .join(".github/maintenance_additions_allowlist.txt")
        .exists());
    assert!(!workspace_root
        .join(".github/root_maintenance_additions_allowlist.txt")
        .exists());
    assert!(!workspace_root
        .join(".github/public_api_allowlist.txt")
        .exists());
}
