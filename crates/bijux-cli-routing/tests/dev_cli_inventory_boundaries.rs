#![forbid(unsafe_code)]
//! Prevents dev-cli route/registry/route-audit presentation assembly from living in routing.

#[test]
fn routing_keeps_only_query_interfaces_for_dev_cli_views() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read routing lib source");

    assert!(
        !source.contains("pub mod reports;"),
        "routing crate must not expose maintainer report assembly modules"
    );
    assert!(
        source.contains("pub mod inventory;"),
        "routing crate must keep read-only inventory query module"
    );
    assert!(
        source.contains("pub mod query;"),
        "routing crate must keep read-only contracts query module"
    );
}
