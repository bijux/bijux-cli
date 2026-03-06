use std::fs;
use std::path::Path;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cargo_toml(path: &str) -> String {
    fs::read_to_string(root().join(path)).expect("read Cargo.toml")
}

#[test]
fn only_cli_crate_declares_bin_target() {
    let crates_dir = root().join("crates");
    let mut offenders = Vec::new();

    for entry in fs::read_dir(&crates_dir).expect("read crates") {
        let entry = entry.expect("crate dir entry");
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let text = fs::read_to_string(&manifest).expect("read manifest");
        let has_bin = text.contains("[[bin]]");
        let is_cli = entry.file_name().to_string_lossy() == "bijux-dag-cli";
        if has_bin && !is_cli {
            offenders.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    assert!(offenders.is_empty(), "non-cli crates declare [[bin]]: {offenders:?}");
}

#[test]
fn core_and_artifacts_do_not_depend_on_clap_or_process_execution_crates() {
    let forbidden = ["clap", "assert_cmd", "duct", "xshell"];
    let manifests = [
        "crates/bijux-dag-core/Cargo.toml",
        "crates/bijux-dag-artifacts/Cargo.toml",
    ];

    for manifest in manifests {
        let text = cargo_toml(manifest);
        for dep in forbidden {
            assert!(
                !text.contains(&format!("{dep} =")),
                "{manifest} must not depend on {dep}"
            );
        }
    }
}

#[test]
fn dev_crate_does_not_depend_on_runtime_crates() {
    let text = cargo_toml("crates/bijux-dev-dag/Cargo.toml");
    for forbidden in ["bijux-dag-runtime", "bijux-dag-app", "bijux-dag-artifacts"] {
        assert!(
            !text.contains(forbidden),
            "bijux-dev-dag must not depend on {forbidden}"
        );
    }
}
