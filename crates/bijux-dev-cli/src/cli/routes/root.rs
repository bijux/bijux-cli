use std::fs;

use bijux_cli::api::version::{runtime_semver, runtime_version};
use serde_json::{json, Value};

use crate::cli::args::{command_has_flag, command_option_value, command_positionals};
use crate::cli::dispatch::RuntimeQueryProvider;
use crate::cli::workspace::workspace_root;
use crate::infra::artifacts::{collect_files_recursive, read_json_if_exists, relative_to_root};
use crate::reports::{
    cockpit as dev_cockpit, contracts as dev_contracts, control_plane as dev_control_plane,
    crate_health as dev_crate_health, docs_audit as dev_docs_audit, env as dev_env,
    maintenance_audit as dev_maintenance_audit, package_health as dev_package_health,
    parity as dev_parity, registry as dev_registry, repo as dev_repo,
    route_audit as dev_route_audit, routes as dev_routes, runtime_identity as dev_runtime_identity,
    state_audit as dev_state_audit, status as dev_status,
};
use crate::schema::command_registry::ReportContext;

pub(super) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    runtime: &dyn RuntimeQueryProvider,
) -> Option<Value> {
    if matches!(normalized_path, [command] if command == "state-doctor") {
        let extra_positionals = command_positionals(argv, &["state-doctor"]);
        if !extra_positionals.is_empty() {
            return Some(json!({
                "status": "error",
                "code": 2,
                "message": "Invalid argument: state-doctor does not accept positional arguments",
                "command": "bijux-dev-cli state-doctor"
            }));
        }
    }

    let payload = match normalized_path {
        [command] if command == "routes" => {
            let context = routing_context();
            let inventory = runtime.route_inventory();
            dev_routes::build_report_from_query(&inventory.routes, &inventory.aliases, &context)
        }
        [command] if command == "atlas" => dev_control_plane::build_atlas_report(),
        [command] if command == "di" => dev_control_plane::build_dependency_injection_report(),
        [command] if command == "list-products" => {
            dev_control_plane::build_product_list_report(&runtime.product_contracts())
        }
        [command] if command == "list-plugins" => {
            dev_control_plane::build_plugin_list_report(runtime.plugin_list())
        }
        [command] if command == "route-audit" => {
            let inventory = runtime.route_inventory();
            dev_route_audit::build_report_from_query(&inventory.routes, &inventory.aliases)
        }
        [command] if command == "inventory" => {
            dev_maintenance_audit::build_inventory_report(&workspace_root())
        }
        [command] if command == "registry" => {
            dev_registry::build_report_from_query(&runtime.registry_inventory(), &routing_context())
        }
        [command] if command == "parity" => dev_parity::build_report(&workspace_root()),
        [command] if command == "docs" => {
            dev_control_plane::build_docs_inventory_report(markdown_docs())
        }
        [command] if command == "status" => {
            let root = workspace_root();
            dev_status::build_report(&root, dev_maintenance_audit::build_inventory_report(&root))
        }
        [command] if command == "maintenance-audit" => {
            let inventory = dev_maintenance_audit::build_inventory_report(&workspace_root());
            dev_maintenance_audit::build_report(inventory)
        }
        [command] if command == "snapshots-audit" => {
            dev_control_plane::build_snapshots_audit_report(snapshot_fixtures())
        }
        [command] if command == "fixture-audit" => {
            let root = workspace_root();
            let parity_files: Vec<String> = collect_files_recursive(&root.join("artifacts/parity"))
                .into_iter()
                .map(|p| relative_to_root(&p, &root))
                .collect();
            dev_control_plane::build_fixture_audit_report(parity_files, snapshot_fixtures())
        }
        [command] if command == "crate-health" => dev_crate_health::build_report(&workspace_root()),
        [command] if command == "package-health" => {
            let root = workspace_root();
            let runtime_identity =
                dev_runtime_identity::build_report(runtime.runtime_identity_input());
            dev_package_health::build_report(&root, runtime_identity)
        }
        [command] if command == "env" => {
            dev_env::build_report(runtime.env_map(), &runtime.active_paths())
        }
        [command] if command == "doctor" => {
            let input = runtime.doctor_report_input();
            dev_control_plane::build_doctor_report(
                input.config_issues,
                input.path_issues,
                input.plugin_issues,
                input.history_issues,
                input.memory_issues,
            )
        }
        [command] if command == "docs-prune-plan" => {
            dev_control_plane::build_docs_prune_plan_report(markdown_docs().len())
        }
        [command] if command == "state-audit" => {
            let input = runtime.state_audit_input();
            dev_state_audit::build_report(input.path_status, input.corruption_health)
        }
        [command] if command == "state-doctor" => {
            dev_state_audit::build_doctor_report(runtime.state_doctor_report())
        }
        [command] if command == "dashboard" => {
            dev_cockpit::build_dashboard_report(&workspace_root())
        }
        [command] if command == "quickcheck" => {
            dev_cockpit::build_quickcheck_report(&workspace_root())
        }
        [command] if command == "truth" => dev_cockpit::build_truth_report(&workspace_root()),
        [command] if command == "blockers" => dev_cockpit::build_blockers_report(&workspace_root()),
        [command] if command == "next" => dev_cockpit::build_next_report(&workspace_root()),
        [command] if command == "docs-audit" => dev_docs_audit::build_report(&workspace_root()),
        [command] if command == "plugin-health" => {
            let root = workspace_root();
            let machine =
                read_json_if_exists(&root.join("artifacts/status/plugin_health_report.json"));
            let text = fs::read_to_string(root.join("artifacts/status/plugin_health_report.txt"))
                .unwrap_or_default();
            dev_control_plane::build_plugin_health_report(machine, text)
        }
        [command] if command == "contracts" => {
            if command_has_flag(argv, &["contracts"], "--all") {
                let kind_filter = command_option_value(argv, &["contracts"], "--kind");
                dev_contracts::build_all_report(
                    &workspace_root(),
                    runtime_version(),
                    kind_filter.as_deref(),
                )
            } else {
                let contracts_query = runtime.contracts_schema_input();
                dev_contracts::build_report_from_query(
                    runtime_semver(),
                    &contracts_query.schema_ids,
                    &contracts_query.schema_version,
                )
            }
        }
        [command] if command == "runtime-identity" => {
            dev_runtime_identity::build_report(runtime.runtime_identity_input())
        }
        [group, command] if group == "repo" && command == "health" => {
            dev_repo::build_health_report(&workspace_root())
        }
        [group, command] if group == "repo" && command == "drift" => {
            dev_repo::build_drift_report(&workspace_root())
        }
        [group, command] if group == "repo" && command == "inventories" => {
            dev_repo::build_inventories_report(&workspace_root())
        }
        [group, command] if group == "repo" && command == "generated" => {
            dev_repo::build_generated_report(&workspace_root())
        }
        [group, command] if group == "repo" && command == "stale" => {
            dev_repo::build_stale_report(&workspace_root())
        }
        _ => return None,
    };

    Some(payload)
}

fn routing_context() -> ReportContext {
    ReportContext { generated_at: String::new(), data_source: "bijux-cli::routing".to_string() }
}

fn markdown_docs() -> Vec<String> {
    let root = workspace_root();
    collect_files_recursive(&root.join("docs"))
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .map(|p| relative_to_root(&p, &root))
        .collect()
}

fn snapshot_fixtures() -> Vec<String> {
    let root = workspace_root();
    collect_files_recursive(&root.join("crates"))
        .into_iter()
        .filter(|p| p.to_string_lossy().contains("tests/data/golden/cli_surface/"))
        .map(|p| relative_to_root(&p, &root))
        .collect()
}
