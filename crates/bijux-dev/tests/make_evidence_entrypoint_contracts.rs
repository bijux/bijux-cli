use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_makefiles(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![root.join("Makefile")];
    let make_dir = root.join("make");
    if make_dir.exists() {
        for entry in fs::read_dir(make_dir).expect("read make dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("mk") {
                files.push(path);
            }
        }
    }
    files
}

#[test]
#[ignore = "legacy make entrypoint contract expects historical module wiring"]
fn root_make_includes_evidence_module() {
    let root = repo_root();
    let root_mk =
        fs::read_to_string(root.join("makes/dag/root.mk")).expect("read makes/dag/root.mk");
    assert!(
        root_mk.contains("include $(DAG_MAKE_DIR)/evidence.mk"),
        "makes/dag/root.mk must include DAG evidence module"
    );
}

#[test]
fn evidence_verify_orchestration_is_only_in_evidence_makefile() {
    let root = repo_root();
    let files = collect_makefiles(&root);
    let mut violations = Vec::new();

    for file in files {
        let rel =
            file.strip_prefix(&root).expect("strip prefix").to_string_lossy().replace('\\', "/");
        let content = fs::read_to_string(&file).expect("read file");

        let has_evidence_verify = content.contains("verify evidence-")
            || content.contains("repo evidence-summary-report");
        if has_evidence_verify && rel != "makes/dag/evidence.mk" {
            violations.push(format!("{rel}: evidence verification command duplication"));
        }

        for line in content.lines() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("evidence-") || !trimmed.contains(':') {
                continue;
            }
            if rel != "makes/dag/evidence.mk" {
                violations
                    .push(format!("{rel}: evidence target defined outside makes/dag/evidence.mk"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "evidence make workflows must be declared in makes/dag/evidence.mk only: {}",
        violations.join(" | ")
    );
}
