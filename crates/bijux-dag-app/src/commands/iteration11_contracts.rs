use serde::{Deserialize, Serialize};

/// Workspace-visible runtime configuration for app routing and execution roots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppWorkspaceConfigV1 {
    pub workspace_id: String,
    pub app_route: String,
    pub run_root: String,
    pub cache_root: String,
    pub plugin_policy: String,
}

/// Resolve workspace behavior surface to prove changes come from visible config.
pub fn resolve_app_workspace_config(
    config: AppWorkspaceConfigV1,
) -> Result<AppWorkspaceConfigV1, String> {
    for (field_name, field_value) in [
        ("workspace_id", config.workspace_id.as_str()),
        ("app_route", config.app_route.as_str()),
        ("run_root", config.run_root.as_str()),
        ("cache_root", config.cache_root.as_str()),
        ("plugin_policy", config.plugin_policy.as_str()),
    ] {
        if field_value.trim().is_empty() {
            return Err(format!("workspace config field {field_name} must not be empty"));
        }
    }
    Ok(config)
}

/// Route inventory diff report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteInventoryDiffV1 {
    pub added_routes: Vec<String>,
    pub removed_routes: Vec<String>,
    pub deprecated_routes: Vec<String>,
    pub conflicted_routes: Vec<String>,
}

/// Diff route inventories between two app versions.
pub fn diff_route_inventory(
    current_routes: Vec<String>,
    next_routes: Vec<String>,
    next_deprecated_routes: Vec<String>,
    conflicts: Vec<String>,
) -> RouteInventoryDiffV1 {
    let mut added_routes = next_routes
        .iter()
        .filter(|route| !current_routes.contains(route))
        .cloned()
        .collect::<Vec<_>>();
    let mut removed_routes = current_routes
        .iter()
        .filter(|route| !next_routes.contains(route))
        .cloned()
        .collect::<Vec<_>>();
    added_routes.sort();
    removed_routes.sort();
    let mut deprecated_routes = next_deprecated_routes;
    deprecated_routes.sort();
    let mut conflicted_routes = conflicts;
    conflicted_routes.sort();
    RouteInventoryDiffV1 {
        added_routes,
        removed_routes,
        deprecated_routes,
        conflicted_routes,
    }
}

/// Compatibility check result for app dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppCompatibilityDecisionV1 {
    pub compatible: bool,
    pub reason: String,
}

/// Enforce host/app compatibility windows before dispatch.
pub fn enforce_app_compatibility_window(
    host_version: u32,
    app_min_supported: u32,
    app_max_supported: u32,
) -> AppCompatibilityDecisionV1 {
    if host_version < app_min_supported {
        return AppCompatibilityDecisionV1 {
            compatible: false,
            reason: "host version is below app minimum supported version".to_string(),
        };
    }
    if host_version > app_max_supported {
        return AppCompatibilityDecisionV1 {
            compatible: false,
            reason: "host version is above app maximum supported version".to_string(),
        };
    }
    AppCompatibilityDecisionV1 {
        compatible: true,
        reason: "host version is within app compatibility window".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        diff_route_inventory, enforce_app_compatibility_window, resolve_app_workspace_config,
        AppWorkspaceConfigV1,
    };

    #[test]
    fn g101_workspace_config_changes_behavior_only_through_visible_fields() {
        let first = resolve_app_workspace_config(AppWorkspaceConfigV1 {
            workspace_id: "project-a".to_string(),
            app_route: "dag".to_string(),
            run_root: "workspace-a/runs".to_string(),
            cache_root: "workspace-a/cache".to_string(),
            plugin_policy: "allow-official".to_string(),
        })
        .expect("workspace a");
        let second = resolve_app_workspace_config(AppWorkspaceConfigV1 {
            workspace_id: "project-b".to_string(),
            app_route: "dag".to_string(),
            run_root: "workspace-b/runs".to_string(),
            cache_root: "workspace-b/cache".to_string(),
            plugin_policy: "allow-official".to_string(),
        })
        .expect("workspace b");
        assert_ne!(first.run_root, second.run_root);
        assert_ne!(first.cache_root, second.cache_root);
    }

    #[test]
    fn g102_route_inventory_diff_exposes_added_removed_deprecated_and_conflicts() {
        let diff = diff_route_inventory(
            vec!["dag run".to_string(), "dag plan".to_string(), "dag old".to_string()],
            vec!["dag run".to_string(), "dag plan".to_string(), "dag inspect".to_string()],
            vec!["dag plan".to_string()],
            vec!["dag inspect".to_string()],
        );
        assert_eq!(diff.added_routes, vec!["dag inspect".to_string()]);
        assert_eq!(diff.removed_routes, vec!["dag old".to_string()]);
        assert_eq!(diff.deprecated_routes, vec!["dag plan".to_string()]);
        assert_eq!(diff.conflicted_routes, vec!["dag inspect".to_string()]);
    }

    #[test]
    fn g103_compatibility_window_blocks_incompatible_apps_before_dispatch() {
        let below = enforce_app_compatibility_window(2, 3, 6);
        assert!(!below.compatible);
        let above = enforce_app_compatibility_window(7, 3, 6);
        assert!(!above.compatible);
        let ok = enforce_app_compatibility_window(4, 3, 6);
        assert!(ok.compatible);
    }
}
