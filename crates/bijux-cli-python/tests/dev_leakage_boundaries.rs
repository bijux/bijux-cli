#![forbid(unsafe_code)]
//! Prevent maintainer workflow leakage into python bridge runtime crate.

#[test]
fn python_bridge_stays_free_of_dev_cli_workflow_assembly() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read python bridge lib");
    assert!(
        !source.contains("bijux_dev_cli"),
        "python bridge runtime crate must not import bijux-dev-cli"
    );
}
