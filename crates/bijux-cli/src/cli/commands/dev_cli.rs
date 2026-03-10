//! `dev cli` command handlers.

use std::env;
use std::fs;
use std::path::Path;

use anyhow::Result;
use bijux_dev_cli::{
    cockpit as dev_cockpit, config as dev_config, contracts as dev_contracts,
    control_plane as dev_control_plane, crate_health as dev_crate_health,
    docs_audit as dev_docs_audit, env as dev_env, evidence as dev_evidence,
    package_health as dev_package_health, parity as dev_parity, python as dev_python,
    registry as dev_registry, release as dev_release, repo as dev_repo,
    route_audit as dev_route_audit, routes as dev_routes, runtime_identity as dev_runtime_identity,
    rustdoc as dev_rustdoc, script_audit as dev_script_audit, scripts as dev_scripts,
    state_audit as dev_state_audit, status as dev_status, ReportContext,
};
use serde_json::{json, Value};

use crate::argv::command_positionals;
use crate::cli::context::{
    collect_files, command_option_value, env_map, read_json_if_exists, rel_to_root,
    state_diagnostics, state_path_status_value, workspace_root, ResolvedStatePaths,
};
use crate::config::storage::{ConfigRepository, FileConfigRepository};
use crate::install::{
    canonical_crate_name, cargo_install_strategy, install_health_report, pip_install_strategy,
    query::runtime_identity_query, InstallHealthReport, PackageChannel,
};
use crate::plugin::{list_plugins, load_time_diagnostics, FUTURE_PRODUCT_NAMESPACES};
use crate::query::state_diagnostics_query;
use crate::routing::inventory::{registry_inventory, route_inventory};
use crate::routing::query::contracts_schema_query;
use crate::routing::registry::RouteRegistry;

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    registry: &RouteRegistry,
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Result<Option<Value>> {
    let payload = match normalized_path {
        [a, b, c] if a == "dev" && b == "cli" && c == "routes" => {
            let context = ReportContext {
                generated_at: String::new(),
                data_source: "bijux-cli::routing".to_string(),
            };
            let inventory = route_inventory(registry);
            dev_routes::build_report_from_query(&inventory.routes, &inventory.aliases, &context)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "atlas" => {
            dev_control_plane::build_atlas_report()
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "di" => {
            dev_control_plane::build_dependency_injection_report()
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "list-products" => {
            dev_control_plane::build_product_list_report(FUTURE_PRODUCT_NAMESPACES)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "list-plugins" => {
            let plugins = list_plugins(plugin_registry_path).unwrap_or_default();
            dev_control_plane::build_plugin_list_report_from(plugins)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "route-audit" => {
            let inventory = route_inventory(registry);
            dev_route_audit::build_report_from_query(&inventory.routes, &inventory.aliases)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "inventory" => {
            dev_script_audit::build_inventory_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "registry" => {
            let context = ReportContext {
                generated_at: String::new(),
                data_source: "bijux-cli::routing".to_string(),
            };
            let inventory = registry_inventory(registry);
            let namespaces: Vec<dev_registry::NamespaceInventoryRow> = inventory
                .namespaces
                .into_iter()
                .map(|row| dev_registry::NamespaceInventoryRow {
                    name: row.name.0,
                    reserved: row.reserved,
                    owner: row.owner,
                })
                .collect();
            dev_registry::build_report_from_query(&namespaces, &context)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "parity" => {
            dev_parity::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs" => {
            let root = workspace_root();
            let docs_files: Vec<String> = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_docs_inventory_report(docs_files)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "status" => dev_status::build_report(
            &workspace_root(),
            dev_script_audit::build_inventory_report(&workspace_root()),
        ),
        [a, b, c] if a == "dev" && b == "cli" && c == "script-audit" => {
            let inventory = dev_script_audit::build_inventory_report(&workspace_root());
            dev_script_audit::build_report(inventory)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "snapshots-audit" => {
            let root = workspace_root();
            let snapshots: Vec<String> = collect_files(&root.join("crates"))
                .into_iter()
                .filter(|p| p.to_string_lossy().contains("tests/snapshots/"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_snapshots_audit_report(snapshots)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "fixture-audit" => {
            let root = workspace_root();
            let parity_files: Vec<String> = collect_files(&root.join("artifacts/parity"))
                .into_iter()
                .map(|p| rel_to_root(&p, &root))
                .collect();
            let snapshots: Vec<String> = collect_files(&root.join("crates"))
                .into_iter()
                .filter(|p| p.to_string_lossy().contains("tests/snapshots/"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_fixture_audit_report(parity_files, snapshots)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "crate-health" => {
            dev_crate_health::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "package-health" => {
            let root = workspace_root();
            let state = read_json_if_exists(&root.join("artifacts/status/current_rust_state.json"));
            dev_package_health::build_report(state)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "env" => dev_env::build_report(
            env_map().into_iter().collect(),
            &dev_env::ActivePaths {
                config_file: paths.config_file.clone(),
                history_file: paths.history_file.clone(),
                plugins_dir: paths.plugins_dir.clone(),
            },
        ),
        [a, b, c] if a == "dev" && b == "cli" && c == "doctor" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let plugin_diagnostics =
                load_time_diagnostics(plugin_registry_path, env!("CARGO_PKG_VERSION"))
                    .unwrap_or_default();
            let repository = FileConfigRepository;
            let config_issues =
                repository.load(&paths.config_file).err().map_or_else(Vec::new, |err| {
                    vec![json!({"category":"config", "message": err.to_string()})]
                });
            let path_issues = if install_report.has_path_shadowing
                || install_report.has_duplicate_installs
            {
                vec![
                    json!({"category":"paths", "has_path_shadowing": install_report.has_path_shadowing}),
                    json!({"category":"paths", "has_duplicate_installs": install_report.has_duplicate_installs}),
                ]
            } else {
                Vec::new()
            };
            let plugin_issues: Vec<Value> = plugin_diagnostics
                .into_iter()
                .map(|diag| {
                    json!({
                        "category": "plugins",
                        "namespace": diag.namespace,
                        "severity": diag.severity,
                        "message": diag.message,
                    })
                })
                .collect();
            dev_control_plane::build_doctor_report(config_issues, path_issues, plugin_issues)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-prune-plan" => {
            let root = workspace_root();
            let docs_count = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .count();
            dev_control_plane::build_docs_prune_plan_report(docs_count)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-audit" => {
            let corruption = state_diagnostics(paths);
            let state_query = state_diagnostics_query(
                &paths.config_file,
                &paths.history_file,
                plugin_registry_path,
                &paths.memory_file,
            );
            dev_state_audit::build_report(
                dev_state_audit::StatePathStatusInput {
                    config: state_path_status_value(&state_query.config),
                    history: state_path_status_value(&state_query.history),
                    plugins_registry: state_path_status_value(&state_query.plugins_registry),
                    memory: state_path_status_value(&state_query.memory),
                },
                corruption,
            )
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-doctor" => {
            let diagnosis = state_diagnostics(paths);
            dev_state_audit::build_doctor_report(diagnosis)
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "remaining" => {
            dev_scripts::build_remaining_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "migrated" => {
            dev_scripts::build_migrated_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "diff" => {
            dev_scripts::build_diff_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "audit" => {
            dev_scripts::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "package-metadata" => {
            dev_scripts::build_package_metadata_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "e2e-contract" => {
            dev_scripts::build_e2e_contract_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "pip-audit" => {
            dev_scripts::build_pip_audit_report(
                &workspace_root(),
                command_option_value(argv, "--report-path").as_deref(),
            )
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "scripts" && d == "capture-python-behavior" =>
        {
            dev_scripts::build_python_capture_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "scripts" && d == "provenance-statement" =>
        {
            let tag = command_option_value(argv, "--tag")
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --tag required"))?;
            let output_dir = command_option_value(argv, "--output-dir")
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --output-dir required"))?;
            dev_scripts::build_provenance_statement_report(&tag, Path::new(&output_dir))
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "audit" => {
            dev_rustdoc::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "coverage" => {
            dev_rustdoc::build_coverage_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "broken-links" => {
            dev_rustdoc::build_broken_links_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "public-api" => {
            dev_rustdoc::build_public_api_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "examples" => {
            dev_rustdoc::build_examples_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "rustdoc" && d == "migrate-website-api-docs" =>
        {
            dev_rustdoc::build_migration_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "build-proof" => {
            dev_rustdoc::build_build_proof_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "rustdoc" && d == "workspace-coverage-proof" =>
        {
            dev_rustdoc::build_workspace_coverage_proof_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "python-link-proof" => {
            dev_rustdoc::build_python_link_proof_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "status" => {
            dev_release::build_status_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "evidence" => {
            dev_release::build_evidence_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "readiness" => {
            dev_release::build_readiness_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "diff" => {
            dev_release::build_diff_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "gaps" => {
            dev_release::build_gaps_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "summary" => {
            dev_release::build_summary_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "manifest" => {
            dev_release::build_manifest_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "notes" => {
            dev_release::build_notes_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "behavior-changes" => {
            dev_release::build_behavior_changes_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "release" && d == "intentional-differences" =>
        {
            dev_release::build_intentional_differences_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "unresolved-gaps" => {
            dev_release::build_unresolved_gaps_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "release" && d == "compatibility-leftovers" =>
        {
            dev_release::build_compatibility_leftovers_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "list" => {
            dev_evidence::build_list_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "show" => {
            let id = command_option_value(argv, "--id")
                .or_else(|| {
                    command_positionals(argv, &["dev", "cli", "evidence", "show"]).first().cloned()
                })
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --id required"))?;
            dev_evidence::build_show_report(&workspace_root(), &id)
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "audit" => {
            dev_evidence::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "stale" => {
            dev_evidence::build_stale_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "matrix" => {
            dev_evidence::build_matrix_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "website-export" => {
            dev_evidence::build_website_export_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "ci-export" => {
            dev_evidence::build_ci_export_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "release-export" => {
            dev_evidence::build_release_export_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "command-map" => {
            dev_evidence::build_command_map_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "parity-map" => {
            dev_evidence::build_parity_map_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "rust-owner" => {
            dev_config::build_rust_owner_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "python-owner" => {
            dev_config::build_python_owner_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "ownership" => {
            dev_config::build_ownership_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "drift" => {
            dev_config::build_drift_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "shape" => {
            dev_config::build_shape_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "evidence-map" => {
            dev_config::build_evidence_map_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "bridge-status" => {
            dev_python::build_bridge_status_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "surface-status" => {
            dev_python::build_surface_status_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "sovereignty-audit" => {
            dev_python::build_sovereignty_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "drift" => {
            dev_python::build_drift_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "packaging" => {
            dev_python::build_packaging_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "health" => {
            dev_repo::build_health_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "drift" => {
            dev_repo::build_drift_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "inventories" => {
            dev_repo::build_inventories_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "generated" => {
            dev_repo::build_generated_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "stale" => {
            dev_repo::build_stale_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "dashboard" => {
            dev_cockpit::build_dashboard_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "quickcheck" => {
            dev_cockpit::build_quickcheck_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "truth" => {
            dev_cockpit::build_truth_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "blockers" => {
            dev_cockpit::build_blockers_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "next" => {
            dev_cockpit::build_next_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-audit" => {
            dev_docs_audit::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "plugin-health" => {
            let root = workspace_root();
            let machine =
                read_json_if_exists(&root.join("artifacts/status/plugin_health_report.json"));
            let text = fs::read_to_string(root.join("artifacts/status/plugin_health_report.txt"))
                .unwrap_or_default();
            dev_control_plane::build_plugin_health_report(machine, text)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "contracts" => {
            let contracts_query = contracts_schema_query();
            dev_contracts::build_report_from_query(
                env!("CARGO_PKG_VERSION"),
                &contracts_query.schema_ids,
                &contracts_query.schema_version,
            )
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "runtime-identity" => {
            let install_query = runtime_identity_query(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let install_report = InstallHealthReport {
                active_binary: install_query.active_binary,
                path_binaries: install_query.path_binaries,
                has_path_shadowing: install_query.has_path_shadowing,
                has_duplicate_installs: install_query.has_duplicate_installs,
                stale_wrapper_scripts: install_query.stale_wrapper_scripts,
                has_mismatched_wheel_binary_versions: install_query
                    .has_mismatched_wheel_binary_versions,
                legacy_installer_conflicts: install_query.legacy_installer_conflicts,
                active_binary_missing: install_query.active_binary_missing,
                broken_symlink_active_binary: install_query.broken_symlink_active_binary,
            };
            let python_bridge_supported = !matches!(
                env::var("BIJUX_PYTHON_BRIDGE_SUPPORTED"),
                Ok(value) if matches!(value.as_str(), "0" | "false" | "FALSE")
            );
            let cargo_canonical = cargo_install_strategy(PackageChannel::Canonical);
            let cargo_compat = cargo_install_strategy(PackageChannel::Compatibility);
            let pip_canonical = pip_install_strategy(PackageChannel::Canonical);
            let pip_compat = pip_install_strategy(PackageChannel::Compatibility);
            dev_runtime_identity::build_report(dev_runtime_identity::RuntimeIdentityInput {
                install_report: dev_runtime_identity::InstallHealthReport {
                    active_binary: install_report.active_binary,
                    path_binaries: install_report.path_binaries,
                    has_path_shadowing: install_report.has_path_shadowing,
                    has_duplicate_installs: install_report.has_duplicate_installs,
                    stale_wrapper_scripts: install_report.stale_wrapper_scripts,
                    has_mismatched_wheel_binary_versions: install_report
                        .has_mismatched_wheel_binary_versions,
                    legacy_installer_conflicts: install_report.legacy_installer_conflicts,
                    active_binary_missing: install_report.active_binary_missing,
                    broken_symlink_active_binary: install_report.broken_symlink_active_binary,
                },
                python_bridge_supported,
                cargo_canonical_package: cargo_canonical.package_name,
                cargo_compat_package: cargo_compat.package_name,
                pip_canonical_package: pip_canonical.package_name,
                pip_compat_package: pip_compat.package_name,
                canonical_crate_name: canonical_crate_name().to_string(),
            })
        }
        _ => return Ok(None),
    };

    Ok(Some(payload))
}
