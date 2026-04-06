#![forbid(unsafe_code)]
//! Architecture boundaries between the runtime binary and maintainer binaries.

use std::path::Path;

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn runtime_dispatch_only_delegates_runtime_product_namespaces() {
    let dispatch = read(concat!(env!("CARGO_MANIFEST_DIR"), "/src/interface/cli/dispatch.rs"));
    let delegation =
        read(concat!(env!("CARGO_MANIFEST_DIR"), "/src/interface/cli/dispatch/delegation.rs"));

    assert!(
        dispatch.contains("try_delegate_known_bijux_tool"),
        "runtime dispatch must keep runtime product delegation at the process boundary"
    );
    assert!(
        delegation.contains("known_bijux_tool"),
        "delegation module must use shared product namespace contracts"
    );
    assert!(
        !delegation.contains("bijux-dev-cli"),
        "runtime delegation must not invoke maintainer binaries directly"
    );
    assert!(
        !dispatch.contains("first == \"dev\""),
        "runtime dispatch must not special-case maintainer command namespaces"
    );
}

#[test]
fn runtime_tests_do_not_require_maintainer_crate_source_layout() {
    let tests_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut stack = vec![tests_root];
    let forbidden =
        format!("..{}bijux-dev-cli{}src", std::path::MAIN_SEPARATOR, std::path::MAIN_SEPARATOR);

    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("failed to read test directory {}: {err}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("failed to read test source {}: {err}", path.display())
            });
            assert!(
                !source.contains(&forbidden),
                "tests must not depend on bijux-dev-cli source tree layout: {}",
                path.display()
            );
        }
    }
}
