#![forbid(unsafe_code)]
//! Ensures routed dev-cli commands remain owned by bijux-dev-cli dispatch modules.

use std::collections::BTreeSet;

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn read_dev_cli_dispatch_source() -> String {
    read(concat!(env!("CARGO_MANIFEST_DIR"), "/../bijux-dev-cli/src/cli/dispatch.rs"))
}

fn read_dev_cli_root_route_source() -> String {
    read(concat!(env!("CARGO_MANIFEST_DIR"), "/../bijux-dev-cli/src/cli/routes/root.rs"))
}

fn extract_guard_values(source: &str, prefix: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = source;
    while let Some(start) = rest.find(prefix) {
        let value_start = start + prefix.len();
        let suffix = &rest[value_start..];
        let Some(value_end) = suffix.find('"') else {
            break;
        };
        out.insert(suffix[..value_end].to_string());
        rest = &suffix[value_end + 1..];
    }
    out
}

fn fixture_dev_cli_top_level_commands() -> BTreeSet<String> {
    let fixture = read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/fixtures/routing/dev_cli_subcommands.txt"
    ));
    fixture
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let segments: Vec<&str> = trimmed.split_whitespace().collect();
            if segments.len() != 3 || segments[0] != "dev" || segments[1] != "cli" {
                return None;
            }
            Some(segments[2].to_string())
        })
        .collect()
}

#[test]
fn dev_cli_subcommand_fixture_matches_dispatch_ownership() {
    let root_source = read_dev_cli_root_route_source();
    let dispatch_source = read_dev_cli_dispatch_source();
    let fixture_commands = fixture_dev_cli_top_level_commands();

    let three_segment_branch_prefix = "[a, b, c] if a == \"dev\" && b == \"cli\" && c == \"";
    let four_segment_namespace_prefix = "[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"";

    let root_three_segment_commands =
        extract_guard_values(&root_source, three_segment_branch_prefix);
    let root_nested_namespaces = extract_guard_values(&root_source, four_segment_namespace_prefix);
    let delegated_namespaces: BTreeSet<String> =
        ["maintenance", "rustdoc", "release", "evidence", "config", "python"]
            .into_iter()
            .map(str::to_string)
            .collect();

    for namespace in &delegated_namespaces {
        let delegate = format!("{namespace}::try_handle");
        assert!(dispatch_source.contains(&delegate), "missing dispatch delegate `{delegate}`");
    }

    let expected_root_three_segment_commands: BTreeSet<String> = fixture_commands
        .difference(&root_nested_namespaces)
        .filter(|command| !delegated_namespaces.contains(*command))
        .cloned()
        .collect();

    assert_eq!(
        root_three_segment_commands, expected_root_three_segment_commands,
        "dev-cli top-level command fixture drifted from root dispatch ownership"
    );
}

#[test]
fn nested_dev_cli_namespaces_have_owned_dispatch_branches() {
    let root_source = read_dev_cli_root_route_source();
    let dispatch_source = read_dev_cli_dispatch_source();

    let branch_prefix = "[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"";
    let observed_root_namespaces = extract_guard_values(&root_source, branch_prefix);
    let expected_root_namespaces: BTreeSet<String> =
        ["repo"].into_iter().map(str::to_string).collect();

    assert_eq!(
        observed_root_namespaces, expected_root_namespaces,
        "root-route namespace ownership drifted from expected architecture"
    );

    for namespace in ["maintenance", "rustdoc", "release", "evidence", "config", "python"] {
        let delegate = format!("{namespace}::try_handle");
        assert!(dispatch_source.contains(&delegate), "namespace `{namespace}` missing delegate");
    }
}

#[test]
fn every_dev_cli_top_level_command_keeps_explicit_delegate_owner() {
    let root_source = read_dev_cli_root_route_source();

    let expected_delegates = [
        ("routes", "dev_routes::build_report_from_query"),
        ("atlas", "dev_control_plane::build_atlas_report"),
        ("di", "dev_control_plane::build_dependency_injection_report"),
        ("list-products", "dev_control_plane::build_product_list_report"),
        ("list-plugins", "dev_control_plane::build_plugin_list_report"),
        ("route-audit", "dev_route_audit::build_report_from_query"),
        ("inventory", "dev_maintenance_audit::build_inventory_report"),
        ("registry", "dev_registry::build_report_from_query"),
        ("parity", "dev_parity::build_report"),
        ("docs", "dev_control_plane::build_docs_inventory_report"),
        ("status", "dev_status::build_report"),
        ("maintenance-audit", "dev_maintenance_audit::build_report"),
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
        assert!(root_source.contains(&branch), "missing route branch for {subcommand}");
        assert!(root_source.contains(delegate), "missing dev-cli delegate for {subcommand}");
    }
}
