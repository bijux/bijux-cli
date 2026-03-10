#![forbid(unsafe_code)]
//! Ensures every routed dev-cli command is implemented by bijux-dev-cli delegates.

#[test]
fn every_dev_cli_subcommand_maps_to_dev_cli_delegate() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("read core app source");

    let expected_delegates = [
        ("routes", "dev_routes::build_report"),
        ("atlas", "dev_control_plane::build_atlas_report"),
        ("di", "dev_control_plane::build_dependency_injection_report"),
        ("list-products", "dev_control_plane::build_product_list_report"),
        ("list-plugins", "dev_control_plane::build_plugin_list_report_from"),
        ("route-audit", "dev_route_audit::build_report"),
        ("inventory", "dev_script_audit::build_inventory_report"),
        ("registry", "dev_registry::build_report"),
        ("parity", "dev_parity::build_report"),
        ("docs", "dev_control_plane::build_docs_inventory_report"),
        ("status", "dev_status::build_report"),
        ("script-audit", "dev_script_audit::build_report"),
        ("snapshots-audit", "dev_control_plane::build_snapshots_audit_report"),
        ("fixture-audit", "dev_control_plane::build_fixture_audit_report"),
        ("crate-health", "dev_crate_health::build_report"),
        ("package-health", "dev_package_health::build_report"),
        ("env", "dev_env::build_report"),
        ("doctor", "dev_control_plane::build_doctor_report"),
        ("contracts", "dev_contracts::build_report_from_query"),
        ("runtime-identity", "dev_runtime_identity::build_report"),
        ("docs-prune-plan", "dev_control_plane::build_docs_prune_plan_report"),
        ("state-audit", "dev_state_audit::build_report"),
        ("state-doctor", "dev_state_audit::build_doctor_report"),
        ("docs-audit", "dev_docs_audit::build_report"),
        ("plugin-health", "dev_control_plane::build_plugin_health_report"),
    ];

    for (subcommand, delegate) in expected_delegates {
        let branch = format!("[a, b, c] if a == \"dev\" && b == \"cli\" && c == \"{subcommand}\"");
        assert!(source.contains(&branch), "missing route branch for {subcommand}");
        assert!(source.contains(delegate), "missing dev-cli delegate for {subcommand}");
    }

    assert!(
        source.contains("[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"scripts\""),
        "missing delegated scripts command namespace"
    );
    assert!(
        source.contains("dev_scripts::build_audit_report"),
        "scripts command namespace must delegate to bijux-dev-cli scripts module"
    );
    assert!(
        source.contains("[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"rustdoc\""),
        "missing delegated rustdoc command namespace"
    );
    assert!(
        source.contains("dev_rustdoc::build_audit_report"),
        "rustdoc command namespace must delegate to bijux-dev-cli rustdoc module"
    );
}
