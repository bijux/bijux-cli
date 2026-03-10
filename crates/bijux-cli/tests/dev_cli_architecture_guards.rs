#![forbid(unsafe_code)]
//! Workspace-level architecture guards for dev-cli ownership.

#[test]
fn runtime_crates_do_not_import_bijux_dev_cli() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let runtime_crates = ["crates/bijux-cli/src/routing", "crates/bijux-cli-python/src"];

    for crate_src in runtime_crates {
        for path in walk_rs_files(&root.join(crate_src)) {
            let source = std::fs::read_to_string(&path).expect("read source file");
            assert!(
                !source.contains("bijux_dev_cli"),
                "runtime crate source must not import bijux-dev-cli: {}",
                path.display()
            );
        }
    }
}

#[test]
fn core_dev_cli_routes_delegate_to_dev_cli_module_helpers() {
    let adapter_source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/features/developer/runtime_adapter.rs"
    ))
    .expect("read dev cli command source");
    let dispatch_source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../bijux-dev-cli/src/dispatch.rs"
    ))
    .expect("read dev cli dispatch source");

    assert!(
        adapter_source.contains("dev_dispatch::try_handle"),
        "core adapter must delegate dev cli command routing to bijux-dev-cli dispatch"
    );
    assert!(
        !adapter_source.contains("dev_control_plane::build_doctor_report"),
        "core adapter must not assemble dev cli report payloads directly"
    );

    let delegated = [
        "dev_control_plane::build_snapshots_audit_report",
        "dev_control_plane::build_fixture_audit_report",
        "dev_control_plane::build_plugin_health_report",
        "dev_control_plane::build_doctor_report",
    ];
    for needle in delegated {
        assert!(dispatch_source.contains(needle), "missing delegation for {needle}");
    }
}

fn walk_rs_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }
    out
}
