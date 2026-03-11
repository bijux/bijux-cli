use serde_json::Value;

use crate::cli::workspace::workspace_root;
use crate::reports::config as dev_config;

pub(super) fn try_handle(normalized_path: &[String]) -> Option<Value> {
    let payload = match normalized_path {
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
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "config" && d == "evidence-map" =>
        {
            dev_config::build_evidence_map_report(&workspace_root())
        }
        _ => return None,
    };

    Some(payload)
}
