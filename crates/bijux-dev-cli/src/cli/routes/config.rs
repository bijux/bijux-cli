use serde_json::Value;

use crate::cli::workspace::workspace_root;
use crate::reports::config as dev_config;

pub(super) fn try_handle(normalized_path: &[String]) -> Option<Value> {
    let payload = match normalized_path {
        [group, command] if group == "config" && command == "rust-owner" => {
            dev_config::build_rust_owner_report(&workspace_root())
        }
        [group, command] if group == "config" && command == "python-owner" => {
            dev_config::build_python_owner_report(&workspace_root())
        }
        [group, command] if group == "config" && command == "ownership" => {
            dev_config::build_ownership_report(&workspace_root())
        }
        [group, command] if group == "config" && command == "drift" => {
            dev_config::build_drift_report(&workspace_root())
        }
        [group, command] if group == "config" && command == "shape" => {
            dev_config::build_shape_report(&workspace_root())
        }
        [group, command] if group == "config" && command == "evidence-map" => {
            dev_config::build_evidence_map_report(&workspace_root())
        }
        _ => return None,
    };

    Some(payload)
}
