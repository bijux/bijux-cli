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

fn target_block<'a>(makefile: &'a str, target: &str) -> &'a str {
    let start = makefile.find(target).unwrap_or_else(|| panic!("missing target {target}"));
    let tail = &makefile[start..];
    let end = tail.find("\n\n").unwrap_or(tail.len());
    &tail[..end]
}

#[test]
fn frozen_gate_entrypoints_delegate_to_pinned_ref_launcher() {
    let makefile = read_repo_file("makes/_internal.mk");

    assert!(
        makefile.contains("PINNED_REF_GATE_BIN  ?= $(ROOT_MK_DIR)/bin/run_pinned_ref_gate.sh"),
        "root make surface must declare the pinned-ref launcher path"
    );

    for (target, gate_target) in [
        ("test-all-frozen:", "PINNED_REF_GATE_TARGET=\"test-all\""),
        ("lint-frozen:", "PINNED_REF_GATE_TARGET=\"lint\""),
        ("audit-frozen:", "PINNED_REF_GATE_TARGET=\"audit\""),
    ] {
        let block = target_block(&makefile, target);
        assert!(
            block.contains("$(PINNED_REF_GATE_BIN)"),
            "{target} must use the shared pinned-ref launcher"
        );
        assert!(
            block.contains(gate_target),
            "{target} must set {gate_target}"
        );
    }
}

#[test]
fn pinned_ref_launcher_isolates_artifacts_and_bootstrap_state() {
    let launcher = read_repo_file("makes/bin/run_pinned_ref_gate.sh");

    for needle in [
        "artifact_root=\"${repo_root}/artifacts/${short_sha}\"",
        "pinned_repo_dir=\"${artifact_root}/frozen-repo\"",
        "background_dir=\"${artifact_root}/background\"",
        "artifact_target_dir=\"${artifact_root}/target/${pinned_target}\"",
        "artifact_cargo_home=\"${artifact_root}/cargo/home/${pinned_target}\"",
        "artifact_tmp_dir=\"${artifact_root}/tmp/${pinned_target}\"",
        "python_venv_dir=\"${python_artifact_root}/.venv\"",
        "python_install_dir=\"${python_artifact_root}/install\"",
        "export VENV=\"${python_venv_dir}\"",
        "export PYTHON_INSTALL_ARTIFACTS_DIR=\"${python_install_dir}\"",
        "console_log=${console_log}",
        "status_file=${status_file}",
    ] {
        assert!(
            launcher.contains(needle),
            "pinned-ref launcher must preserve `{needle}`"
        );
    }
}

#[test]
fn frozen_gate_docs_publish_usage_contract() {
    let ci_targets = read_repo_file("docs/bijux-dev/makes/ci-targets.md");
    let root_entrypoints = read_repo_file("docs/bijux-dev/makes/root-entrypoints.md");

    for needle in [
        "TEST_ALL_FROZEN_REF=<ref> make test-all-frozen",
        "TEST_ALL_FROZEN_REF=<ref> make lint-frozen",
        "TEST_ALL_FROZEN_REF=<ref> make audit-frozen",
        "artifacts/<sha>/frozen-repo/",
        "artifacts/<sha>/background/",
        "artifacts/<sha>/python/",
    ] {
        assert!(
            ci_targets.contains(needle),
            "CI targets handbook must document `{needle}`"
        );
    }

    for needle in ["make test-all-frozen", "make lint-frozen", "make audit-frozen"] {
        assert!(
            root_entrypoints.contains(needle),
            "root entrypoints handbook must advertise `{needle}`"
        );
    }
}
