use serde_json::Value;

use crate::cli::workspace::workspace_root;
use crate::reports::python as dev_python;

pub(super) fn try_handle(normalized_path: &[String]) -> Option<Value> {
    let payload = match normalized_path {
        [group, command] if group == "python" && command == "bridge-status" => {
            dev_python::build_bridge_status_report(&workspace_root())
        }
        [group, command] if group == "python" && command == "surface-status" => {
            dev_python::build_surface_status_report(&workspace_root())
        }
        [group, command] if group == "python" && command == "sovereignty-audit" => {
            dev_python::build_sovereignty_audit_report(&workspace_root())
        }
        [group, command] if group == "python" && command == "drift" => {
            dev_python::build_drift_report(&workspace_root())
        }
        [group, command] if group == "python" && command == "packaging" => {
            dev_python::build_packaging_report(&workspace_root())
        }
        _ => return None,
    };

    Some(payload)
}
