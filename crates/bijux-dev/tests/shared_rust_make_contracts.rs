use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

fn collect_rust_sources(path: &Path, output: &mut String) {
    for entry in fs::read_dir(path).expect("read Rust source directory") {
        let entry = entry.expect("read Rust source entry");
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_rust_sources(&entry_path, output);
        } else if entry_path.extension().and_then(|value| value.to_str()) == Some("rs") {
            output.push_str(&fs::read_to_string(entry_path).expect("read Rust source"));
            output.push('\n');
        }
    }
}

#[test]
fn root_make_entrypoint_loads_shared_common_and_rust_contracts() {
    let root = repo_root();
    let root_make = fs::read_to_string(root.join("makes/root.mk")).expect("read root Make module");
    let rust_make = fs::read_to_string(root.join("makes/rust.mk")).expect("read Rust Make policy");

    for expected in
        ["bijux-makes/environment.mk", "bijux-makes/guards.mk", "bijux-makes-rs/bijux.mk"]
    {
        assert!(root_make.contains(expected), "root Make module must load {expected}");
    }
    for expected in [
        "RS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT_ABS)/rust",
        "RS_RUN_ID ?= $(RUN_ID)",
        "NEXTEST_SLOW_NAME_EXPR ?= test(/::slow__/)",
        "RUST_GATE_BIN ?= $(CORE_RUST_GATE_BIN)",
        "RUST_AUDIT_PREREQUISITES += audit-policy-rs",
        "NEXTEST_PROFILE_FAST=\"$(NEXTEST_RELEASE_PROFILE)\" \"$(RUST_GATE_BIN)\" test",
    ] {
        assert!(rust_make.contains(expected), "Rust Make policy must declare {expected}");
    }
}

#[test]
fn core_adapter_retains_binary_preparation_and_delegates_execution() {
    let root = repo_root();
    let adapter = fs::read_to_string(root.join("makes/bin/run_core_rust_gate.sh"))
        .expect("read Core Rust gate adapter");

    for expected in [
        ".bijux/shared/bijux-makes-rs/scripts/rust_gate.sh",
        "cargo build --locked -p bijux-dev --bin bijux-dev-cli",
        "cargo build --locked -p bijux-dag-cli --bin bijux-dag",
        "exec \"${shared_gate}\" \"$@\"",
    ] {
        assert!(adapter.contains(expected), "Core adapter must retain {expected}");
    }
    assert!(!root.join("makes/bin/run_pinned_ref_gate.sh").exists());
}

#[test]
fn complete_lane_includes_ignored_tests_and_disables_retries() {
    let shared_gate =
        fs::read_to_string(repo_root().join(".bijux/shared/bijux-makes-rs/scripts/rust_gate.sh"))
            .expect("read shared Rust gate");
    assert!(shared_gate.contains("args+=(--run-ignored all --retries 0)"));
}

#[test]
fn governed_roster_is_sorted_unique_and_resolves_to_slow_tests() {
    let root = repo_root();
    let roster_content = fs::read_to_string(root.join("configs/rust/nextest-slow-roster.txt"))
        .expect("read slow-test roster");
    let roster = roster_content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    let sorted_unique = roster.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        roster,
        sorted_unique.into_iter().collect::<Vec<_>>(),
        "slow-test roster must remain sorted and unique"
    );

    let mut sources = String::new();
    collect_rust_sources(&root.join("crates"), &mut sources);
    for test_name in roster {
        assert!(
            sources.contains(&format!("fn {test_name}(")),
            "slow-test roster entry does not resolve to a Rust test: {test_name}"
        );
        assert!(
            !test_name.contains("slow__"),
            "slow__ tests must not be duplicated in the measured roster: {test_name}"
        );
    }

    assert!(
        !sources.lines().any(|line| {
            let line = line.trim_start();
            line.starts_with("fn slow_") && !line.starts_with("fn slow__")
                || line.starts_with("async fn slow_") && !line.starts_with("async fn slow__")
        }),
        "explicitly slow Rust tests must use the slow__ namespace"
    );
}
