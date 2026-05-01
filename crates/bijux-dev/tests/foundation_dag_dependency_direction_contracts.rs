use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct DagDependencyContract {
    schema_version: String,
    crates: Vec<DagDependencyRow>,
}

#[derive(Debug, Deserialize)]
struct DagDependencyRow {
    #[serde(rename = "crate")]
    crate_name: String,
    allowed_workspace_normal_deps: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_contract() -> DagDependencyContract {
    let path = repo_root().join("contracts/foundation/dag_dependency_direction.v1.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw).expect("dag dependency direction contract must be valid JSON")
}

fn workspace_normal_deps_by_crate() -> BTreeMap<String, BTreeSet<String>> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(repo_root())
        .output()
        .expect("run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");
    let payload: Value = serde_json::from_slice(&output.stdout).expect("parse metadata json");
    let packages = payload["packages"].as_array().expect("packages array");

    packages
        .iter()
        .filter_map(|pkg| {
            let name = pkg["name"].as_str()?;
            if !name.starts_with("bijux-dag-") {
                return None;
            }
            let deps = pkg["dependencies"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|dep| dep["kind"].is_null())
                .filter_map(|dep| dep["name"].as_str())
                .filter(|dep_name| dep_name.starts_with("bijux-dag-"))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>();
            Some((name.to_string(), deps))
        })
        .collect()
}

fn rs_files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn strip_comments_and_strings(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut idx = 0;
    let mut block_comment_depth = 0_usize;
    let mut in_line_comment = false;
    let mut in_string = false;
    let mut in_raw_string_hashes: Option<usize> = None;
    let mut escaped = false;

    while idx < bytes.len() {
        let current = bytes[idx];
        let next = bytes.get(idx + 1).copied();

        if in_line_comment {
            if current == b'\n' {
                in_line_comment = false;
                out.push('\n');
            }
            idx += 1;
            continue;
        }

        if block_comment_depth > 0 {
            if current == b'/' && next == Some(b'*') {
                block_comment_depth += 1;
                idx += 2;
                continue;
            }
            if current == b'*' && next == Some(b'/') {
                block_comment_depth -= 1;
                idx += 2;
                continue;
            }
            if current == b'\n' {
                out.push('\n');
            }
            idx += 1;
            continue;
        }

        if let Some(raw_hashes) = in_raw_string_hashes {
            if current == b'"' {
                let mut matches = true;
                for offset in 0..raw_hashes {
                    if bytes.get(idx + 1 + offset).copied() != Some(b'#') {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    in_raw_string_hashes = None;
                    idx += 1 + raw_hashes;
                    out.push(' ');
                    continue;
                }
            }
            if current == b'\n' {
                out.push('\n');
            }
            idx += 1;
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
                idx += 1;
                continue;
            }
            if current == b'\\' {
                escaped = true;
                idx += 1;
                continue;
            }
            if current == b'"' {
                in_string = false;
                out.push(' ');
                idx += 1;
                continue;
            }
            if current == b'\n' {
                out.push('\n');
            }
            idx += 1;
            continue;
        }

        if current == b'/' && next == Some(b'/') {
            in_line_comment = true;
            idx += 2;
            continue;
        }
        if current == b'/' && next == Some(b'*') {
            block_comment_depth = 1;
            idx += 2;
            continue;
        }

        if current == b'r' {
            let mut hashes = 0_usize;
            let mut cursor = idx + 1;
            while bytes.get(cursor).copied() == Some(b'#') {
                hashes += 1;
                cursor += 1;
            }
            if bytes.get(cursor).copied() == Some(b'"') {
                in_raw_string_hashes = Some(hashes);
                idx = cursor + 1;
                out.push(' ');
                continue;
            }
        }

        if current == b'"' {
            in_string = true;
            out.push(' ');
            idx += 1;
            continue;
        }

        out.push(current as char);
        idx += 1;
    }

    out
}

fn assert_no_forbidden_import_tokens(root: &Path, forbidden: &[&str]) {
    let mut offenders = Vec::new();
    for file in rs_files_under(root) {
        let source = fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        let cleaned = strip_comments_and_strings(&source);
        if forbidden.iter().any(|token| cleaned.contains(token)) {
            offenders.push(file.display().to_string());
        }
    }
    assert!(offenders.is_empty(), "forbidden DAG import tokens detected: {offenders:?}");
}

#[test]
fn dag_workspace_dependencies_follow_the_contract_direction() {
    let contract = read_contract();
    assert_eq!(contract.schema_version, "foundation-dag-dependency-direction/v1");

    let expected = contract
        .crates
        .into_iter()
        .map(|row| {
            (row.crate_name, row.allowed_workspace_normal_deps.into_iter().collect::<BTreeSet<_>>())
        })
        .collect::<BTreeMap<_, _>>();
    let observed = workspace_normal_deps_by_crate();

    assert_eq!(
        observed, expected,
        "DAG crate dependency direction drifted from foundation contract"
    );
}

#[test]
fn dag_source_layers_do_not_import_higher_level_dag_crates() {
    let root = repo_root();
    assert_no_forbidden_import_tokens(
        &root.join("crates/bijux-dag-core/src"),
        &["bijux_dag_runtime::", "bijux_dag_app::", "bijux_dag_cli::", "bijux_dag_testkit::"],
    );
    assert_no_forbidden_import_tokens(
        &root.join("crates/bijux-dag-artifacts/src"),
        &["bijux_dag_runtime::", "bijux_dag_app::", "bijux_dag_cli::"],
    );
    assert_no_forbidden_import_tokens(
        &root.join("crates/bijux-dag-runtime/src"),
        &["bijux_dag_app::", "bijux_dag_cli::"],
    );
}
