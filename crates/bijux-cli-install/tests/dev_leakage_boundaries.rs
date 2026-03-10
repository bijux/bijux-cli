#![forbid(unsafe_code)]
//! Prevent maintainer workflow leakage into install runtime crate.

#[test]
fn install_runtime_stays_free_of_dev_cli_workflow_assembly() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read install lib");
    assert!(
        !source.contains("bijux_dev_cli"),
        "install runtime crate must not import bijux-dev-cli"
    );
}
