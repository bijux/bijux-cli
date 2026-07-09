use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize)]
struct UnimplementedSurfacePolicy {
    forbidden_tokens: Vec<String>,
    forbidden_public_output_tokens: Vec<String>,
    allowed_public_output_exceptions: Vec<UnimplementedSurfaceException>,
}

#[derive(Debug, serde::Deserialize)]
struct UnimplementedSurfaceException {
    path: String,
    contains: String,
    reason: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default();
                if matches!(name, "target" | "artifacts" | ".git") {
                    continue;
                }
                stack.push(path);
            } else if path.extension().and_then(|v| v.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
}

fn collect_json_files(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|v| v.to_str()) == Some("json") {
                out.push(path);
            }
        }
    }
}

fn is_excluded(rel: &str) -> bool {
    rel.contains("/tests/") || rel.contains("/benches/") || rel.ends_with(".in.rs")
}

#[test]
fn stable_sources_reject_todo_and_unimplemented_markers() {
    let root = repo_root();
    let policy: UnimplementedSurfacePolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/unimplemented_surface_policy.json"))
            .expect("read unimplemented surface policy"),
    )
    .expect("parse unimplemented surface policy");

    let mut files = Vec::new();
    collect_rs_files(&root.join("crates"), &mut files);

    let mut violations = Vec::new();
    for file in files {
        let rel =
            file.strip_prefix(&root).expect("strip prefix").to_string_lossy().replace('\\', "/");
        if is_excluded(&rel) {
            continue;
        }
        let content = fs::read_to_string(&file).expect("read file");
        for token in &policy.forbidden_tokens {
            if content.contains(token) {
                violations.push(format!("{} -> {}", rel, token));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "stable source includes forbidden unimplemented-surface markers: {}",
        violations.join(" | ")
    );
}

#[test]
fn public_output_unimplemented_text_requires_policy_exception() {
    let root = repo_root();
    let policy: UnimplementedSurfacePolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/unimplemented_surface_policy.json"))
            .expect("read unimplemented surface policy"),
    )
    .expect("parse unimplemented surface policy");

    let exceptions: Vec<(String, String)> = policy
        .allowed_public_output_exceptions
        .iter()
        .map(|entry| (entry.path.clone(), entry.contains.clone()))
        .collect();

    let mut files = Vec::new();
    collect_rs_files(&root.join("crates"), &mut files);

    let mut violations = Vec::new();
    for file in files {
        let rel =
            file.strip_prefix(&root).expect("strip prefix").to_string_lossy().replace('\\', "/");
        if rel.contains("/tests/") || rel.contains("/benches/") {
            continue;
        }
        if !(rel.starts_with("crates/bijux-dag-cli/")
            || rel.starts_with("crates/bijux-dag-app/")
            || rel.starts_with("crates/bijux-dag-artifacts/"))
        {
            continue;
        }
        let content = fs::read_to_string(&file).expect("read file");

        for token in &policy.forbidden_public_output_tokens {
            if !content.contains(token) {
                continue;
            }
            for line in content.lines() {
                if !line.contains(token) {
                    continue;
                }
                let allowed =
                    exceptions.iter().any(|(path, snippet)| path == &rel && line.contains(snippet));
                if !allowed {
                    violations.push(format!("{} -> {}", rel, line.trim()));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "public output unimplemented-surface text needs explicit exception: {}",
        violations.join(" | ")
    );
}

#[test]
fn release_blocking_evidence_assets_are_unimplemented_surface_free() {
    let root = repo_root();
    let release: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/release/release_evidence_set.json"))
            .expect("read release evidence set"),
    )
    .expect("parse release evidence set");

    let mut violations = Vec::new();
    let blocking_assets = release
        .get("blocking_assets")
        .and_then(serde_json::Value::as_array)
        .expect("blocking assets array");

    for asset in blocking_assets {
        let rel = asset.as_str().expect("blocking asset path");
        let path = root.join(rel);
        let text = fs::read_to_string(&path).expect("read blocking asset");
        for token in ["placeholder", "TODO", "TBD"] {
            if text.contains(token) {
                violations.push(format!("{} -> {}", rel, token));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "release-blocking evidence asset contains unimplemented-surface markers: {}",
        violations.join(" | ")
    );
}

#[test]
fn battle_scenarios_are_unimplemented_surface_free() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_json_files(&root.join("evidence/battle/workflows"), &mut files);

    let mut violations = Vec::new();
    for file in files {
        let rel =
            file.strip_prefix(&root).expect("strip prefix").to_string_lossy().replace('\\', "/");
        let text = fs::read_to_string(&file).expect("read battle scenario");
        for token in ["placeholder", "TODO", "TBD"] {
            if text.contains(token) {
                violations.push(format!("{} -> {}", rel, token));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "battle scenario contains unimplemented-surface marker: {}",
        violations.join(" | ")
    );
}

#[test]
fn unimplemented_surface_exceptions_must_have_reasons() {
    let root = repo_root();
    let policy: UnimplementedSurfacePolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/unimplemented_surface_policy.json"))
            .expect("read unimplemented surface policy"),
    )
    .expect("parse unimplemented surface policy");

    for exception in policy.allowed_public_output_exceptions {
        assert!(
            !exception.reason.trim().is_empty(),
            "unimplemented surface exception missing reason: {}",
            exception.path
        );
    }
}

#[test]
fn operator_command_surfaces_are_unimplemented_surface_free() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_rs_files(&root.join("crates/bijux-dag-app/src"), &mut files);

    let mut violations = Vec::new();
    for file in files {
        let rel =
            file.strip_prefix(&root).expect("strip prefix").to_string_lossy().replace('\\', "/");
        if !(rel.contains("/commands/") || rel.ends_with("src/lib.rs")) {
            continue;
        }
        let text = fs::read_to_string(&file).expect("read operator surface file");
        for token in ["not implemented", "Not implemented", "placeholder"] {
            if text.contains(token) {
                violations.push(format!("{} -> {}", rel, token));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "operator command surface contains unimplemented-surface wording: {}",
        violations.join(" | ")
    );
}
