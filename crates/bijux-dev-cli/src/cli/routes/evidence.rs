use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::cli::args::{command_option_value, command_positionals};
use crate::cli::workspace::workspace_root;
use crate::reports::evidence as dev_evidence;

pub(super) fn try_handle(normalized_path: &[String], argv: &[String]) -> Result<Option<Value>> {
    let payload = match normalized_path {
        [group, command] if group == "evidence" && command == "list" => {
            dev_evidence::build_list_report(&workspace_root())
        }
        [group, command] if group == "evidence" && command == "show" => {
            let id = command_option_value(argv, &["evidence", "show"], "--id")
                .or_else(|| command_positionals(argv, &["evidence", "show"]).first().cloned())
                .ok_or_else(|| anyhow!("Missing argument: --id required"))?;
            dev_evidence::build_show_report(&workspace_root(), &id)
        }
        [group, command] if group == "evidence" && command == "audit" => {
            dev_evidence::build_audit_report(&workspace_root())
        }
        [group, command] if group == "evidence" && command == "stale" => {
            dev_evidence::build_stale_report(&workspace_root())
        }
        [group, command] if group == "evidence" && command == "matrix" => {
            dev_evidence::build_matrix_report(&workspace_root())
        }
        [group, command] if group == "evidence" && command == "website-export" => {
            dev_evidence::build_website_export_report(&workspace_root())
        }
        [group, command] if group == "evidence" && command == "ci-export" => {
            dev_evidence::build_ci_export_report(&workspace_root())
        }
        [group, command] if group == "evidence" && command == "release-export" => {
            dev_evidence::build_release_export_report(&workspace_root())
        }
        [group, command] if group == "evidence" && command == "command-map" => {
            dev_evidence::build_command_map_report(&workspace_root())
        }
        [group, command] if group == "evidence" && command == "parity-map" => {
            dev_evidence::build_parity_map_report(&workspace_root())
        }
        _ => return Ok(None),
    };

    Ok(Some(payload))
}
