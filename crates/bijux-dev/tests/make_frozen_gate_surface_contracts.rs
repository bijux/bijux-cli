use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    let absolute = repo_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

#[test]
fn frozen_gate_entrypoints_delegate_to_pinned_ref_launcher() {
    let makefile = read_repo_file(".bijux/shared/bijux-makes-rs/cargo.mk");

    for (target, gate_target) in [
        ("test-all-frozen:", "PINNED_GATE_TARGET=test-all"),
        ("lint-frozen:", "PINNED_GATE_TARGET=lint"),
        ("audit-frozen:", "PINNED_GATE_TARGET=audit"),
    ] {
        assert!(
            makefile.contains(target) && makefile.contains(gate_target),
            "{target} must use the shared pinned launcher with {gate_target}"
        );
    }
}

#[test]
fn pinned_ref_launcher_isolates_artifacts_and_bootstrap_state() {
    let launcher = read_repo_file(".bijux/shared/bijux-makes/scripts/run_pinned_gate.sh");

    for needle in [
        "artifact_root=\"${repo_root}/artifacts/${short_sha}\"",
        "pinned_repo_dir=\"${artifact_root}/frozen-repo\"",
        "background_dir=\"${artifact_root}/background\"",
        "export ARTIFACT_ROOT=\"${artifact_root}\"",
        "export RUN_ID=\"${short_sha}\"",
        "console_log=${console_log}",
        "status_file=${status_file}",
    ] {
        assert!(launcher.contains(needle), "pinned-ref launcher must preserve `{needle}`");
    }
}

#[test]
fn frozen_gate_docs_publish_usage_contract() {
    let ci_targets = read_repo_file("docs/bijux-dev/makes/ci-targets.md");
    let root_entrypoints = read_repo_file("docs/bijux-dev/makes/root-entrypoints.md");

    for needle in [
        "PINNED_REF=<ref> make test-all-frozen",
        "PINNED_REF=<ref> make lint-frozen",
        "PINNED_REF=<ref> make audit-frozen",
        "artifacts/<sha>/frozen-repo/",
        "artifacts/<sha>/background/",
        "artifacts/<sha>/rust/",
    ] {
        assert!(ci_targets.contains(needle), "CI targets handbook must document `{needle}`");
    }

    for needle in ["make test-all-frozen", "make lint-frozen", "make audit-frozen"] {
        assert!(
            root_entrypoints.contains(needle),
            "root entrypoints handbook must advertise `{needle}`"
        );
    }
}
