#![forbid(unsafe_code)]
//! Prevents dev-cli route/registry/route-audit presentation assembly from living in routing.

#[test]
fn routing_keeps_only_query_interfaces_for_dev_cli_views() {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/routing/mod.rs"))
            .expect("read routing module source");

    assert!(
        !source.contains("pub mod reports;"),
        "routing module must not expose maintainer report assembly modules"
    );
    assert!(
        !source.contains("pub mod inventory;"),
        "routing module must not expose route inventory query interfaces"
    );
    assert!(
        !source.contains("pub mod query;"),
        "routing module must not expose contract schema query interfaces"
    );
}
