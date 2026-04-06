use serde_json::Value;

use crate::cli::workspace::workspace_root;
use crate::reports::release as dev_release;

pub(super) fn try_handle(normalized_path: &[String]) -> Option<Value> {
    let payload = match normalized_path {
        [group, command] if group == "release" && command == "status" => {
            dev_release::build_status_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "evidence" => {
            dev_release::build_evidence_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "readiness" => {
            dev_release::build_readiness_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "diff" => {
            dev_release::build_diff_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "gaps" => {
            dev_release::build_gaps_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "summary" => {
            dev_release::build_summary_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "manifest" => {
            dev_release::build_manifest_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "notes" => {
            dev_release::build_notes_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "behavior-changes" => {
            dev_release::build_behavior_changes_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "intentional-differences" => {
            dev_release::build_intentional_differences_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "unresolved-gaps" => {
            dev_release::build_unresolved_gaps_report(&workspace_root())
        }
        [group, command] if group == "release" && command == "compatibility-leftovers" => {
            dev_release::build_compatibility_leftovers_report(&workspace_root())
        }
        _ => return None,
    };

    Some(payload)
}
