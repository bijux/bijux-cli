use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::cli::args::{command_option_value, command_passthrough_args};
use crate::cli::workspace::workspace_root;
use crate::contracts::maintenance as dev_maintenance;

pub(super) fn try_handle(normalized_path: &[String], argv: &[String]) -> Result<Option<Value>> {
    let payload = match normalized_path {
        [group, command] if group == "maintenance" && command == "remaining" => {
            dev_maintenance::build_remaining_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "migrated" => {
            dev_maintenance::build_migrated_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "diff" => {
            dev_maintenance::build_diff_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "audit" => {
            dev_maintenance::build_audit_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "generators" => {
            dev_maintenance::build_generators_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "generate" => {
            let source_ref =
                command_option_value(argv, &["maintenance", "generate"], "--source-ref").or_else(
                    || command_option_value(argv, &["maintenance", "generate"], "--source"),
                );
            dev_maintenance::run_generator(
                &workspace_root(),
                command_option_value(argv, &["maintenance", "generate"], "--id").as_deref(),
                source_ref.as_deref(),
            )
        }
        [group, command] if group == "maintenance" && command == "generate-all" => {
            dev_maintenance::run_all_generators(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "requirements" => {
            dev_maintenance::build_requirement_catalog_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "flaky-tests" => {
            dev_maintenance::build_flaky_tests_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "ignored-dag-tests" => {
            dev_maintenance::build_ignored_dag_tests_report(&workspace_root())
        }
        [group, section, command]
            if group == "maintenance" && section == "status" && command == "inventory" =>
        {
            dev_maintenance::build_status_contracts_report(&workspace_root())
        }
        [group, section, command]
            if group == "maintenance" && section == "status" && command == "run" =>
        {
            let passthrough = command_passthrough_args(argv, &["maintenance", "status", "run"]);
            let source_ref =
                command_option_value(argv, &["maintenance", "status", "run"], "--source-ref")
                    .or_else(|| {
                        command_option_value(argv, &["maintenance", "status", "run"], "--source")
                    });
            dev_maintenance::run_status_contract(
                &workspace_root(),
                command_option_value(argv, &["maintenance", "status", "run"], "--id").as_deref(),
                source_ref.as_deref(),
                &passthrough,
            )
        }
        [group, section, command]
            if group == "maintenance" && section == "status" && command == "run-all" =>
        {
            let passthrough = command_passthrough_args(argv, &["maintenance", "status", "run-all"]);
            dev_maintenance::run_all_status_contracts(
                &workspace_root(),
                command_option_value(argv, &["maintenance", "status", "run-all"], "--kind")
                    .as_deref(),
                &passthrough,
            )
        }
        [group, command] if group == "maintenance" && command == "package-metadata" => {
            dev_maintenance::build_package_metadata_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "e2e-contract" => {
            dev_maintenance::build_e2e_contract_report(&workspace_root())
        }
        [group, command] if group == "maintenance" && command == "pip-audit" => {
            dev_maintenance::build_pip_audit_report(
                &workspace_root(),
                command_option_value(argv, &["maintenance", "pip-audit"], "--report-path")
                    .as_deref(),
            )
        }
        [group, command] if group == "maintenance" && command == "provenance-statement" => {
            let tag = command_option_value(argv, &["maintenance", "provenance-statement"], "--tag")
                .ok_or_else(|| anyhow!("Missing argument: --tag required"))?;
            let output_dir = command_option_value(
                argv,
                &["maintenance", "provenance-statement"],
                "--output-dir",
            )
            .ok_or_else(|| anyhow!("Missing argument: --output-dir required"))?;
            dev_maintenance::build_provenance_statement_report(&tag, Path::new(&output_dir))
        }
        _ => return Ok(None),
    };

    Ok(Some(payload))
}
