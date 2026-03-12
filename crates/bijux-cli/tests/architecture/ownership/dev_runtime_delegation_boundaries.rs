#![forbid(unsafe_code)]
//! Architecture boundaries for delegated `dev` and product mount commands.

use std::path::Path;

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn runtime_dispatch_keeps_dev_and_product_execution_at_process_boundary() {
    let dispatch = read(concat!(env!("CARGO_MANIFEST_DIR"), "/src/interface/cli/dispatch.rs"));
    let delegation =
        read(concat!(env!("CARGO_MANIFEST_DIR"), "/src/interface/cli/dispatch/delegation.rs"));

    assert!(
        dispatch.contains("try_delegate_known_bijux_tool"),
        "runtime dispatch must delegate `dev` and product namespaces before local handlers"
    );
    assert!(
        delegation.contains("delegate_dev_cli"),
        "delegation module must call the external dev control-plane binary"
    );
    assert!(
        delegation.contains("known_bijux_tool"),
        "delegation module must use shared product namespace contracts"
    );
    assert!(
        !delegation.contains("bijux_dev_cli"),
        "runtime crate must not import dev-cli crate symbols directly"
    );
}

#[test]
fn bijux_cli_tests_do_not_require_bijux_dev_cli_source_layout() {
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
