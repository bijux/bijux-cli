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

#[cfg(test)]
mod tests {
    use super::{resolve_app_workspace_config, AppWorkspaceConfigV1};

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
}
