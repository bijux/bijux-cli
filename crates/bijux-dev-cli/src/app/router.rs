//! Dev-cli command routing.

use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::app::args::{
    command_has_flag, command_option_value, command_passthrough_args, command_positionals,
};
use crate::app::runtime_query::RuntimeQueryProvider;
use crate::app::workspace::workspace_root;
use crate::reports::{
    cockpit as dev_cockpit, config as dev_config, control_plane as dev_control_plane,
    crate_health as dev_crate_health, docs_audit as dev_docs_audit, env as dev_env,
    evidence as dev_evidence, maintenance_audit as dev_maintenance_audit,
    package_health as dev_package_health, parity as dev_parity, python as dev_python,
    registry as dev_registry, release as dev_release, repo as dev_repo,
    route_audit as dev_route_audit, routes as dev_routes, contracts as dev_contracts,
    runtime_identity as dev_runtime_identity, rustdoc as dev_rustdoc,
    state_audit as dev_state_audit, status as dev_status,
};
use crate::infrastructure::artifacts::{
    collect_files_recursive, read_json_if_exists, relative_to_root,
};
use crate::{maintenance as dev_maintenance, ReportContext};

/// Return true when the normalized path belongs to `dev cli` dispatch ownership.
#[must_use]
pub fn owns_path(normalized_path: &[String]) -> bool {
    match normalized_path {
        [a, b, _] if a == "dev" && b == "cli" => true,
        [a, b, c, _]
            if a == "dev"
                && b == "cli"
                && matches!(
                    c.as_str(),
                    "maintenance"
                        | "rustdoc"
                        | "release"
                        | "evidence"
                        | "config"
                        | "python"
                        | "repo"
                ) =>
        {
            true
        }
        [a, b, c, d, _] if a == "dev" && b == "cli" && c == "maintenance" && d == "status" => true,
        _ => false,
    }
}

/// Dispatch `dev cli` command paths and return report payloads.
pub fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    runtime: &dyn RuntimeQueryProvider,
) -> Result<Option<Value>> {
    let payload = match normalized_path {
        [a, b, c] if a == "dev" && b == "cli" && c == "routes" => {
            let context = ReportContext {
                generated_at: String::new(),
                data_source: "bijux-cli::routing".to_string(),
            };
            let inventory = runtime.route_inventory();
            dev_routes::build_report_from_query(&inventory.routes, &inventory.aliases, &context)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "atlas" => {
            dev_control_plane::build_atlas_report()
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "di" => {
            dev_control_plane::build_dependency_injection_report()
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "list-products" => {
            dev_control_plane::build_product_list_report(&runtime.product_contracts())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "list-plugins" => {
            dev_control_plane::build_plugin_list_report(runtime.plugin_list())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "route-audit" => {
            let inventory = runtime.route_inventory();
            dev_route_audit::build_report_from_query(&inventory.routes, &inventory.aliases)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "inventory" => {
            dev_maintenance_audit::build_inventory_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "registry" => {
            let context = ReportContext {
                generated_at: String::new(),
                data_source: "bijux-cli::routing".to_string(),
            };
            dev_registry::build_report_from_query(&runtime.registry_inventory(), &context)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "parity" => {
            dev_parity::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs" => {
            let root = workspace_root();
            let docs_files: Vec<String> = collect_files_recursive(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .map(|p| relative_to_root(&p, &root))
                .collect();
            dev_control_plane::build_docs_inventory_report(docs_files)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "status" => dev_status::build_report(
            &workspace_root(),
            dev_maintenance_audit::build_inventory_report(&workspace_root()),
        ),
        [a, b, c] if a == "dev" && b == "cli" && c == "maintenance-audit" => {
            let inventory = dev_maintenance_audit::build_inventory_report(&workspace_root());
            dev_maintenance_audit::build_report(inventory)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "snapshots-audit" => {
            let root = workspace_root();
            let snapshots: Vec<String> = collect_files_recursive(&root.join("crates"))
                .into_iter()
                .filter(|p| {
                    p.to_string_lossy()
                        .contains("tests/data/golden/cli_surface/")
                })
                .map(|p| relative_to_root(&p, &root))
                .collect();
            dev_control_plane::build_snapshots_audit_report(snapshots)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "fixture-audit" => {
            let root = workspace_root();
            let parity_files: Vec<String> = collect_files_recursive(&root.join("artifacts/parity"))
                .into_iter()
                .map(|p| relative_to_root(&p, &root))
                .collect();
            let snapshots: Vec<String> = collect_files_recursive(&root.join("crates"))
                .into_iter()
                .filter(|p| {
                    p.to_string_lossy()
                        .contains("tests/data/golden/cli_surface/")
                })
                .map(|p| relative_to_root(&p, &root))
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
        [a, b, c] if a == "dev" && b == "cli" && c == "env" => {
            dev_env::build_report(runtime.env_map(), &runtime.active_paths())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "doctor" => {
            let input = runtime.doctor_report_input();
            dev_control_plane::build_doctor_report(
                input.config_issues,
                input.path_issues,
                input.plugin_issues,
            )
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-prune-plan" => {
            let root = workspace_root();
            let docs_count = collect_files_recursive(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .count();
            dev_control_plane::build_docs_prune_plan_report(docs_count)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-audit" => {
            let input = runtime.state_audit_input();
            dev_state_audit::build_report(input.path_status, input.corruption_health)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-doctor" => {
            dev_state_audit::build_doctor_report(runtime.state_doctor_report())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "remaining" => {
            dev_maintenance::build_remaining_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "migrated" => {
            dev_maintenance::build_migrated_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "diff" => {
            dev_maintenance::build_diff_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "audit" => {
            dev_maintenance::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "generators" => {
            dev_maintenance::build_generators_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "generate" => {
            let source_ref = command_option_value(argv, "--source-ref")
                .or_else(|| command_option_value(argv, "--source"));
            dev_maintenance::run_generator(
                &workspace_root(),
                command_option_value(argv, "--id").as_deref(),
                source_ref.as_deref(),
            )
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "generate-all" => {
            dev_maintenance::run_all_generators(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "requirements" => {
            dev_maintenance::build_requirement_catalog_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "flaky-tests" => {
            dev_maintenance::build_flaky_tests_report(&workspace_root())
        }
        [a, b, c, d, e]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && d == "status"
                && e == "inventory" =>
        {
            dev_maintenance::build_status_maintenance_report(&workspace_root())
        }
        [a, b, c, d, e]
            if a == "dev" && b == "cli" && c == "maintenance" && d == "status" && e == "run" =>
        {
            let passthrough = command_passthrough_args(argv);
            let source_ref = command_option_value(argv, "--source-ref")
                .or_else(|| command_option_value(argv, "--source"));
            dev_maintenance::run_status_contract(
                &workspace_root(),
                command_option_value(argv, "--id").as_deref(),
                source_ref.as_deref(),
                &passthrough,
            )
        }
        [a, b, c, d, e]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && d == "status"
                && e == "run-all" =>
        {
            let passthrough = command_passthrough_args(argv);
            dev_maintenance::run_all_status_maintenance(
                &workspace_root(),
                command_option_value(argv, "--kind").as_deref(),
                &passthrough,
            )
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "maintenance" && d == "package-metadata" =>
        {
            dev_maintenance::build_package_metadata_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "e2e-contract" => {
            dev_maintenance::build_e2e_contract_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "maintenance" && d == "pip-audit" => {
            dev_maintenance::build_pip_audit_report(
                &workspace_root(),
                command_option_value(argv, "--report-path").as_deref(),
            )
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "maintenance" && d == "capture-python-behavior" =>
        {
            dev_maintenance::build_python_capture_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "maintenance" && d == "provenance-statement" =>
        {
            let tag = command_option_value(argv, "--tag")
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --tag required"))?;
            let output_dir = command_option_value(argv, "--output-dir")
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --output-dir required"))?;
            dev_maintenance::build_provenance_statement_report(&tag, Path::new(&output_dir))
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
                    command_positionals(argv, &["dev", "cli", "evidence", "show"])
                        .first()
                        .cloned()
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
            if command_has_flag(argv, "--all") {
                let kind_filter = command_option_value(argv, "--kind");
                dev_contracts::build_all_report(
                    &workspace_root(),
                    env!("CARGO_PKG_VERSION"),
                    kind_filter.as_deref(),
                )
            } else {
                let contracts_query = runtime.contracts_schema_input();
                dev_contracts::build_report_from_query(
                    env!("CARGO_PKG_VERSION"),
                    &contracts_query.schema_ids,
                    &contracts_query.schema_version,
                )
            }
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "runtime-identity" => {
            dev_runtime_identity::build_report(runtime.runtime_identity_input())
        }
        _ => return Ok(None),
    };

    Ok(Some(payload))
}
