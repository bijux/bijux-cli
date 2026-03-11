#![forbid(unsafe_code)]
//! Crate-level command group contracts for bijux-dev-cli control-plane assembly.

use std::path::Path;

use bijux_dev_cli::{
    cockpit, config, contracts, control_plane, crate_health, docs_audit, env, package_health,
    parity, python, registry, repo, route_audit, routes, runtime_identity, script_audit,
    state_audit, status, ReportContext,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn all_command_groups_build_expected_top_level_keys() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let context = ReportContext {
        generated_at: "now".to_string(),
        data_source: "tests".to_string(),
    };
    let routes_rows = vec![vec![
        "dev".to_string(),
        "cli".to_string(),
        "status".to_string(),
    ]];
    let alias_rows = vec![(
        vec!["status".to_string()],
        vec!["cli".to_string(), "status".to_string()],
    )];
    let namespace_rows = vec![registry::NamespaceInventoryRow {
        name: "dev".to_string(),
        reserved: true,
        owner: "bijux-cli".to_string(),
    }];

    assert!(
        routes::build_report_from_query(&routes_rows, &alias_rows, &context)
            .get("routes")
            .is_some()
    );
    assert!(registry::build_report_from_query(&namespace_rows, &context)
        .get("registry")
        .is_some());
    assert!(
        route_audit::build_report_from_query(&routes_rows, &alias_rows)
            .get("summary")
            .is_some()
    );
    assert!(env::build_report(
        BTreeMap::new(),
        &env::ActivePaths {
            config_file: "/tmp/config".into(),
            history_file: "/tmp/history".into(),
            plugins_dir: "/tmp/plugins".into(),
        }
    )
    .get("active")
    .is_some());
    assert!(contracts::build_report("0.1.0").get("contracts").is_some());
    assert!(config::build_ownership_report(&root)
        .get("owners")
        .is_some());
    assert!(python::build_sovereignty_audit_report(&root)
        .get("python_sovereignty_audit")
        .is_some());
    assert!(repo::build_health_report(&root)
        .get("repo_health")
        .is_some());
    assert!(cockpit::build_dashboard_report(&root)
        .get("dashboard")
        .is_some());
    assert!(parity::build_report(&root).get("command_matrix").is_some());
    assert!(
        status::build_report(&root, script_audit::build_inventory_report(&root))
            .get("status_report")
            .is_some()
    );
    assert!(
        runtime_identity::build_report(runtime_identity::RuntimeIdentityInput {
            install_report: runtime_identity::InstallHealthReport {
                active_binary: None,
                path_binaries: vec![],
                has_path_shadowing: false,
                has_duplicate_installs: false,
                stale_wrapper_scripts: vec![],
                has_mismatched_wheel_binary_versions: false,
                legacy_installer_conflicts: vec![],
                active_binary_missing: false,
                broken_symlink_active_binary: false,
            },
            python_bridge_supported: true,
            cargo_canonical_package: "bijux-cli".to_string(),
            cargo_compat_package: "bijux".to_string(),
            pip_canonical_package: "bijux-cli".to_string(),
            pip_compat_package: "bijux".to_string(),
            canonical_crate_name: "bijux-cli".to_string(),
        })
        .get("entrypoints")
        .is_some()
    );
    assert!(package_health::build_report(json!({}))
        .get("install_state_assumptions")
        .is_some());
    assert!(state_audit::build_report(
        state_audit::StatePathStatusInput {
            config: json!({}),
            history: json!({}),
            plugins_registry: json!({}),
            memory: json!({}),
        },
        json!({})
    )
    .get("paths")
    .is_some());
    assert!(docs_audit::build_report(&root).get("docs_count").is_some());
    let inventory = script_audit::build_inventory_report(&root);
    assert!(script_audit::build_report(inventory)
        .get("scripts")
        .is_some());
    assert!(crate_health::build_report(&root)
        .get("crate_metrics")
        .is_some());
    assert!(control_plane::build_atlas_report().get("mount").is_some());
}
