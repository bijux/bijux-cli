use serde_json::Value;

use crate::cli::workspace::workspace_root;
use crate::reports::python as dev_python;

pub(super) fn try_handle(normalized_path: &[String]) -> Option<Value> {
    let payload = match normalized_path {
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
        _ => return None,
    };

    Some(payload)
}
