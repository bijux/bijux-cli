#![forbid(unsafe_code)]
//! Ensures every routed dev-cli command is implemented in bijux-dev-cli dispatch.

#[test]
fn every_dev_cli_subcommand_maps_to_dev_cli_delegate() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../bijux-dev-cli/src/dispatch.rs"
    ))
    .expect("read dev cli dispatch source");

    let expected_delegates = [
        ("routes", "dev_routes::build_report_from_query"),
        ("atlas", "dev_control_plane::build_atlas_report"),
        ("di", "dev_control_plane::build_dependency_injection_report"),
        ("list-products", "dev_control_plane::build_product_list_report"),
        ("list-plugins", "dev_control_plane::build_plugin_list_report"),
        ("route-audit", "dev_route_audit::build_report_from_query"),
        ("inventory", "dev_script_audit::build_inventory_report"),
        ("registry", "dev_registry::build_report_from_query"),
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
        ("dashboard", "dev_cockpit::build_dashboard_report"),
        ("quickcheck", "dev_cockpit::build_quickcheck_report"),
        ("truth", "dev_cockpit::build_truth_report"),
        ("blockers", "dev_cockpit::build_blockers_report"),
        ("next", "dev_cockpit::build_next_report"),
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
    assert!(
        source.contains("[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"release\""),
        "missing delegated release command namespace"
    );
    assert!(
        source.contains("dev_release::build_status_report"),
        "release command namespace must delegate to bijux-dev-cli release module"
    );
    assert!(
        source.contains("[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"evidence\""),
        "missing delegated evidence command namespace"
    );
    assert!(
        source.contains("dev_evidence::build_list_report"),
        "evidence command namespace must delegate to bijux-dev-cli evidence module"
    );
    assert!(
        source.contains("[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"config\""),
        "missing delegated config command namespace"
    );
    assert!(
        source.contains("dev_config::build_ownership_report"),
        "config command namespace must delegate to bijux-dev-cli config module"
    );
    assert!(
        source.contains("[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"python\""),
        "missing delegated python command namespace"
    );
    assert!(
        source.contains("dev_python::build_sovereignty_audit_report"),
        "python command namespace must delegate to bijux-dev-cli python module"
    );
    assert!(
        source.contains("[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"repo\""),
        "missing delegated repo command namespace"
    );
    assert!(
        source.contains("dev_repo::build_health_report"),
        "repo command namespace must delegate to bijux-dev-cli repo module"
    );
}
