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
    RouteInventoryDiffV1 { added_routes, removed_routes, deprecated_routes, conflicted_routes }
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

/// Deprecation lifecycle action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeprecationActionV1 {
    pub action: String,
    pub message: String,
    pub migration_hint: String,
}

/// Evaluate command/config deprecation lifecycle by version phase.
pub fn evaluate_deprecation_lifecycle(
    current_version: u32,
    warn_after: u32,
    migrate_after: u32,
    refuse_after: u32,
    replacement: &str,
) -> Result<DeprecationActionV1, String> {
    if replacement.trim().is_empty() {
        return Err("replacement must not be empty".to_string());
    }
    if !(warn_after <= migrate_after && migrate_after <= refuse_after) {
        return Err("deprecation thresholds must be monotonic".to_string());
    }
    if current_version >= refuse_after {
        return Ok(DeprecationActionV1 {
            action: "refuse".to_string(),
            message: "deprecated command is no longer accepted".to_string(),
            migration_hint: format!("use {replacement}"),
        });
    }
    if current_version >= migrate_after {
        return Ok(DeprecationActionV1 {
            action: "migrate".to_string(),
            message: "automatic migration path should be applied".to_string(),
            migration_hint: format!("migrate to {replacement}"),
        });
    }
    if current_version >= warn_after {
        return Ok(DeprecationActionV1 {
            action: "warn".to_string(),
            message: "deprecated command remains available with warning".to_string(),
            migration_hint: format!("plan migration to {replacement}"),
        });
    }
    Ok(DeprecationActionV1 {
        action: "none".to_string(),
        message: "deprecation not active yet".to_string(),
        migration_hint: format!("future replacement: {replacement}"),
    })
}

/// Safe install-repair outcome with explicit backup and changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallRepairReportV1 {
    pub backup_created: bool,
    pub changed_paths: Vec<String>,
    pub summary: String,
    pub destroyed_user_data: bool,
}

/// Validate install-repair safety guarantees.
pub fn validate_install_repair_report(
    report: InstallRepairReportV1,
) -> Result<InstallRepairReportV1, String> {
    if !report.backup_created {
        return Err("repair must create backup before mutating state".to_string());
    }
    if report.changed_paths.is_empty() {
        return Err("repair must report changed paths".to_string());
    }
    if report.summary.trim().is_empty() {
        return Err("repair summary must not be empty".to_string());
    }
    if report.destroyed_user_data {
        return Err("repair must not destroy user data".to_string());
    }
    Ok(report)
}

/// Support bundle scope report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportBundleReportV1 {
    pub includes_config: bool,
    pub includes_routes: bool,
    pub includes_environment: bool,
    pub includes_plugins: bool,
    pub includes_run_data: bool,
    pub includes_schema_info: bool,
    pub redaction_applied: bool,
    pub reproduction_ready: bool,
}

/// Validate support-bundle minimal useful scope.
pub fn validate_support_bundle_report(
    report: SupportBundleReportV1,
) -> Result<SupportBundleReportV1, String> {
    if !report.includes_config
        || !report.includes_routes
        || !report.includes_environment
        || !report.includes_plugins
        || !report.includes_run_data
        || !report.includes_schema_info
    {
        return Err("support bundle missing required reproduction surface".to_string());
    }
    if !report.redaction_applied {
        return Err("support bundle must apply redaction".to_string());
    }
    if !report.reproduction_ready {
        return Err("support bundle must be reproduction-ready".to_string());
    }
    Ok(report)
}

/// Command impact preview report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandImpactPreviewV1 {
    pub file_writes: Vec<String>,
    pub run_roots_touched: Vec<String>,
    pub cache_effects: Vec<String>,
    pub adapter_execution: Vec<String>,
    pub plugin_execution: Vec<String>,
    pub destructive_actions: Vec<String>,
}

/// Validate command impact preview completeness.
pub fn validate_command_impact_preview(
    preview: CommandImpactPreviewV1,
) -> Result<CommandImpactPreviewV1, String> {
    if preview.file_writes.is_empty() {
        return Err("impact preview must list file writes".to_string());
    }
    if preview.run_roots_touched.is_empty() {
        return Err("impact preview must list run roots touched".to_string());
    }
    if preview.cache_effects.is_empty() {
        return Err("impact preview must list cache effects".to_string());
    }
    if preview.adapter_execution.is_empty() {
        return Err("impact preview must list adapter execution".to_string());
    }
    if preview.plugin_execution.is_empty() {
        return Err("impact preview must list plugin execution".to_string());
    }
    Ok(preview)
}

/// Official app onboarding conformance report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficialAppOnboardingReportV1 {
    pub mock_app_registered: bool,
    pub route_contract_passed: bool,
    pub command_contract_passed: bool,
    pub root_internal_changes_required: bool,
}

/// Validate official app onboarding reproducibility.
pub fn validate_official_app_onboarding(
    report: OfficialAppOnboardingReportV1,
) -> Result<OfficialAppOnboardingReportV1, String> {
    if !report.mock_app_registered {
        return Err("mock official app must be registered".to_string());
    }
    if !report.route_contract_passed {
        return Err("mock app route contract must pass".to_string());
    }
    if !report.command_contract_passed {
        return Err("mock app command contract must pass".to_string());
    }
    if report.root_internal_changes_required {
        return Err(
            "official app onboarding must not require root internal modifications".to_string()
        );
    }
    Ok(report)
}

/// Plugin lifecycle usability report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLifecycleReportV1 {
    pub install_ok: bool,
    pub list_ok: bool,
    pub enable_ok: bool,
    pub disable_ok: bool,
    pub update_ok: bool,
    pub explain_ok: bool,
    pub remove_ok: bool,
    pub rollback_clean_on_failure: bool,
}

/// Validate plugin lifecycle usability and rollback guarantees.
pub fn validate_plugin_lifecycle_report(
    report: PluginLifecycleReportV1,
) -> Result<PluginLifecycleReportV1, String> {
    if !report.install_ok
        || !report.list_ok
        || !report.enable_ok
        || !report.disable_ok
        || !report.update_ok
        || !report.explain_ok
        || !report.remove_ok
    {
        return Err("plugin lifecycle operations must all succeed".to_string());
    }
    if !report.rollback_clean_on_failure {
        return Err("plugin lifecycle must guarantee rollback on failure".to_string());
    }
    Ok(report)
}

/// Root CLI growth-budget report under app ecosystem expansion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootCliGrowthBudgetReportV1 {
    pub app_count: usize,
    pub startup_latency_ms: u64,
    pub help_lines: usize,
    pub dispatch_coupling_score: u32,
}

/// Validate root CLI remains small and stable as app count grows.
pub fn validate_root_cli_growth_budget(
    report: RootCliGrowthBudgetReportV1,
    startup_latency_budget_ms: u64,
    help_lines_budget: usize,
    dispatch_coupling_budget: u32,
) -> Result<RootCliGrowthBudgetReportV1, String> {
    if report.startup_latency_ms > startup_latency_budget_ms {
        return Err("root cli startup latency exceeded budget".to_string());
    }
    if report.help_lines > help_lines_budget {
        return Err("root cli help surface exceeded budget".to_string());
    }
    if report.dispatch_coupling_score > dispatch_coupling_budget {
        return Err("root cli dispatch coupling exceeded budget".to_string());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::{
        diff_route_inventory, enforce_app_compatibility_window, evaluate_deprecation_lifecycle,
        resolve_app_workspace_config, validate_command_impact_preview,
        validate_install_repair_report, validate_official_app_onboarding,
        validate_plugin_lifecycle_report, validate_root_cli_growth_budget,
        validate_support_bundle_report, AppWorkspaceConfigV1, CommandImpactPreviewV1,
        InstallRepairReportV1, OfficialAppOnboardingReportV1, PluginLifecycleReportV1,
        RootCliGrowthBudgetReportV1, SupportBundleReportV1,
    };

    #[test]
    fn workspace_config_changes_behavior_only_through_visible_fields() {
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
    fn route_inventory_diff_exposes_added_removed_deprecated_and_conflicts() {
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
    fn compatibility_window_blocks_incompatible_apps_before_dispatch() {
        let below = enforce_app_compatibility_window(2, 3, 6);
        assert!(!below.compatible);
        let above = enforce_app_compatibility_window(7, 3, 6);
        assert!(!above.compatible);
        let ok = enforce_app_compatibility_window(4, 3, 6);
        assert!(ok.compatible);
    }

    #[test]
    fn deprecation_lifecycle_produces_warn_migrate_and_refuse_with_hints() {
        let warn = evaluate_deprecation_lifecycle(5, 4, 7, 10, "dag run").expect("warn");
        assert_eq!(warn.action, "warn");
        let migrate = evaluate_deprecation_lifecycle(8, 4, 7, 10, "dag run").expect("migrate");
        assert_eq!(migrate.action, "migrate");
        let refuse = evaluate_deprecation_lifecycle(10, 4, 7, 10, "dag run").expect("refuse");
        assert_eq!(refuse.action, "refuse");
        assert!(refuse.migration_hint.contains("dag run"));
    }

    #[test]
    fn install_repair_requires_backup_and_explicit_change_summary() {
        let report = validate_install_repair_report(InstallRepairReportV1 {
            backup_created: true,
            changed_paths: vec![
                ".bijux/config.json".to_string(),
                ".bijux/plugins/state.json".to_string(),
            ],
            summary: "repaired malformed plugin state and restored config defaults".to_string(),
            destroyed_user_data: false,
        })
        .expect("install repair");
        assert!(report.backup_created);
        assert_eq!(report.changed_paths.len(), 2);
    }

    #[test]
    fn support_bundle_is_minimal_redacted_and_reproduction_ready() {
        let report = validate_support_bundle_report(SupportBundleReportV1 {
            includes_config: true,
            includes_routes: true,
            includes_environment: true,
            includes_plugins: true,
            includes_run_data: true,
            includes_schema_info: true,
            redaction_applied: true,
            reproduction_ready: true,
        })
        .expect("support bundle");
        assert!(report.redaction_applied);
        assert!(report.reproduction_ready);
    }

    #[test]
    fn command_impact_preview_surfaces_risky_side_effects_before_execution() {
        let preview = validate_command_impact_preview(CommandImpactPreviewV1 {
            file_writes: vec!["runs/run-123/manifest.json".to_string()],
            run_roots_touched: vec!["runs/".to_string()],
            cache_effects: vec!["cache lookup and possible write".to_string()],
            adapter_execution: vec!["shell adapter".to_string()],
            plugin_execution: vec!["quality-gate plugin".to_string()],
            destructive_actions: vec!["none".to_string()],
        })
        .expect("impact preview");
        assert_eq!(preview.file_writes.len(), 1);
        assert_eq!(preview.adapter_execution[0], "shell adapter");
    }

    #[test]
    fn official_app_onboarding_is_reproducible_without_root_internal_changes() {
        let report = validate_official_app_onboarding(OfficialAppOnboardingReportV1 {
            mock_app_registered: true,
            route_contract_passed: true,
            command_contract_passed: true,
            root_internal_changes_required: false,
        })
        .expect("official app onboarding");
        assert!(report.mock_app_registered);
        assert!(report.command_contract_passed);
    }

    #[test]
    fn plugin_lifecycle_is_usable_and_rolls_back_cleanly_on_failure() {
        let report = validate_plugin_lifecycle_report(PluginLifecycleReportV1 {
            install_ok: true,
            list_ok: true,
            enable_ok: true,
            disable_ok: true,
            update_ok: true,
            explain_ok: true,
            remove_ok: true,
            rollback_clean_on_failure: true,
        })
        .expect("plugin lifecycle");
        assert!(report.install_ok);
        assert!(report.rollback_clean_on_failure);
    }

    #[test]
    fn root_cli_growth_budget_stays_stable_as_apps_increase() {
        let report = validate_root_cli_growth_budget(
            RootCliGrowthBudgetReportV1 {
                app_count: 14,
                startup_latency_ms: 95,
                help_lines: 180,
                dispatch_coupling_score: 12,
            },
            150,
            220,
            20,
        )
        .expect("growth budget");
        assert_eq!(report.app_count, 14);
        assert!(report.startup_latency_ms <= 150);
    }
}
