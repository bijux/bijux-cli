#![forbid(unsafe_code)]
//! Ensures routing crate stays command-identity-only for dev-cli surfaces.

#[test]
fn parser_and_registry_do_not_assemble_dev_cli_reports() {
    let routing_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bijux-cli/src/routing");
    let parser = std::fs::read_to_string(routing_root.join("parser.rs")).expect("read parser.rs");
    let registry =
        std::fs::read_to_string(routing_root.join("registry.rs")).expect("read registry.rs");

    assert!(!parser.contains("build_report("), "parser must not host maintainer report assembly");
    assert!(
        !registry.contains("build_report("),
        "registry must not host maintainer report assembly"
    );
    assert!(
        !parser.contains("serde_json::json!"),
        "parser must remain route identity and parse intent only"
    );
    assert!(
        !registry.contains("serde_json::json!"),
        "registry must remain route identity and resolution only"
    );
}
