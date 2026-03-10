#![forbid(unsafe_code)]
//! Prevent maintainer workflow leakage into output runtime crate.

#[test]
fn output_runtime_stays_free_of_dev_cli_workflow_assembly() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read output lib");
    assert!(
        !source.contains("bijux_dev_cli"),
        "output runtime crate must not import bijux-dev-cli"
    );
    assert!(
        !source.contains("dev cli"),
        "output runtime crate must not hardcode maintainer command workflows"
    );
}
