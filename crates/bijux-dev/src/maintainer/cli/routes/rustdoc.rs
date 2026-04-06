use serde_json::Value;

use crate::cli::workspace::workspace_root;
use crate::reports::rustdoc as dev_rustdoc;

pub(super) fn try_handle(normalized_path: &[String]) -> Option<Value> {
    let payload = match normalized_path {
        [group, command] if group == "rustdoc" && command == "audit" => {
            dev_rustdoc::build_audit_report(&workspace_root())
        }
        [group, command] if group == "rustdoc" && command == "coverage" => {
            dev_rustdoc::build_coverage_report(&workspace_root())
        }
        [group, command] if group == "rustdoc" && command == "broken-links" => {
            dev_rustdoc::build_broken_links_report(&workspace_root())
        }
        [group, command] if group == "rustdoc" && command == "public-api" => {
            dev_rustdoc::build_public_api_report(&workspace_root())
        }
        [group, command] if group == "rustdoc" && command == "examples" => {
            dev_rustdoc::build_examples_report(&workspace_root())
        }
        [group, command] if group == "rustdoc" && command == "migrate-website-api-docs" => {
            dev_rustdoc::build_migration_report(&workspace_root())
        }
        [group, command] if group == "rustdoc" && command == "build-proof" => {
            dev_rustdoc::build_build_proof_report(&workspace_root())
        }
        [group, command] if group == "rustdoc" && command == "workspace-coverage-proof" => {
            dev_rustdoc::build_workspace_coverage_proof_report(&workspace_root())
        }
        [group, command] if group == "rustdoc" && command == "python-link-proof" => {
            dev_rustdoc::build_python_link_proof_report(&workspace_root())
        }
        _ => return None,
    };

    Some(payload)
}
