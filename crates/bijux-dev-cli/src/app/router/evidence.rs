use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::app::args::{command_option_value, command_positionals};
use crate::app::workspace::workspace_root;
use crate::reports::evidence as dev_evidence;

pub(super) fn try_handle(normalized_path: &[String], argv: &[String]) -> Result<Option<Value>> {
    let payload = match normalized_path {
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
                .ok_or_else(|| anyhow!("Missing argument: --id required"))?;
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
        _ => return Ok(None),
    };

    Ok(Some(payload))
}
