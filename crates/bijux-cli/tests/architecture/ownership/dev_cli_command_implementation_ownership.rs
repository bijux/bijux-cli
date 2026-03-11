#![forbid(unsafe_code)]
//! Ensures every routed dev-cli command is implemented in bijux-dev-cli dispatch.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn read_dev_cli_router_source() -> String {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let router_root = crate_root.join("../bijux-dev-cli/src/app/router");
    assert!(
        router_root.is_dir(),
        "expected modular dev-cli router directory at {}",
        router_root.display()
    );
    let mut files = Vec::<PathBuf>::new();
    collect_rs_files(&router_root, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "expected dev-cli router source files under {}",
        router_root.display()
    );

    let mut source = String::new();
    for file in files {
        source.push_str(&read(&file.to_string_lossy()));
        source.push('\n');
    }
    source
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
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
fn dev_cli_subcommand_fixture_exactly_matches_three_segment_dispatch_surface() {
    let source = read_dev_cli_router_source();
    let fixture_commands = fixture_dev_cli_top_level_commands();
    let three_segment_branch_prefix = "[a, b, c] if a == \"dev\" && b == \"cli\" && c == \"";
    let four_segment_namespace_prefix = "[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"";

    let dispatch_three_segment_commands =
        extract_guard_values(&source, three_segment_branch_prefix);
    let nested_namespaces = extract_guard_values(&source, four_segment_namespace_prefix);

    let expected_three_segment_commands: BTreeSet<String> = fixture_commands
        .difference(&nested_namespaces)
        .cloned()
        .collect();

    assert_eq!(
        dispatch_three_segment_commands, expected_three_segment_commands,
        "dev-cli top-level command fixture drifted from dispatch ownership"
    );
}

#[test]
fn nested_dev_cli_namespaces_have_owned_dispatch_branches() {
    let source = read_dev_cli_router_source();

    let branch_prefix = "[a, b, c, d] if a == \"dev\" && b == \"cli\" && c == \"";
    let observed_namespaces = extract_guard_values(&source, branch_prefix);
    let expected_namespaces: BTreeSet<String> = [
        "maintenance",
        "rustdoc",
        "release",
        "evidence",
        "config",
        "python",
        "repo",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        observed_namespaces, expected_namespaces,
        "nested namespace dispatch ownership drifted from expected architecture"
    );

    let expected_delegate_prefixes = BTreeMap::from([
        ("maintenance", "dev_maintenance::build_"),
        ("rustdoc", "dev_rustdoc::build_"),
        ("release", "dev_release::build_"),
        ("evidence", "dev_evidence::build_"),
        ("config", "dev_config::build_"),
        ("python", "dev_python::build_"),
        ("repo", "dev_repo::build_"),
    ]);
    for (namespace, delegate_prefix) in expected_delegate_prefixes {
        assert!(
            source.contains(delegate_prefix),
            "namespace `{namespace}` must delegate to `{delegate_prefix}*` implementations"
        );
    }
}

#[test]
fn every_dev_cli_top_level_command_keeps_explicit_delegate_owner() {
    let source = read_dev_cli_router_source();

    let expected_delegates = [
        ("routes", "dev_routes::build_report_from_query"),
        ("atlas", "dev_control_plane::build_atlas_report"),
        ("di", "dev_control_plane::build_dependency_injection_report"),
        (
            "list-products",
            "dev_control_plane::build_product_list_report",
        ),
        (
            "list-plugins",
            "dev_control_plane::build_plugin_list_report",
        ),
        ("route-audit", "dev_route_audit::build_report_from_query"),
        ("inventory", "dev_maintenance_audit::build_inventory_report"),
        ("registry", "dev_registry::build_report_from_query"),
        ("parity", "dev_parity::build_report"),
        ("docs", "dev_control_plane::build_docs_inventory_report"),
        ("status", "dev_status::build_report"),
        ("maintenance-audit", "dev_maintenance_audit::build_report"),
        (
            "snapshots-audit",
            "dev_control_plane::build_snapshots_audit_report",
        ),
        (
            "fixture-audit",
            "dev_control_plane::build_fixture_audit_report",
        ),
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
        (
            "docs-prune-plan",
            "dev_control_plane::build_docs_prune_plan_report",
        ),
        ("state-audit", "dev_state_audit::build_report"),
        ("state-doctor", "dev_state_audit::build_doctor_report"),
        ("docs-audit", "dev_docs_audit::build_report"),
        (
            "plugin-health",
            "dev_control_plane::build_plugin_health_report",
        ),
    ];

    for (subcommand, delegate) in expected_delegates {
        let branch = format!("[a, b, c] if a == \"dev\" && b == \"cli\" && c == \"{subcommand}\"");
        assert!(
            source.contains(&branch),
            "missing route branch for {subcommand}"
        );
        assert!(
            source.contains(delegate),
            "missing dev-cli delegate for {subcommand}"
        );
    }
}
