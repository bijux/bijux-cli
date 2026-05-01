use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Compact root help entrypoint contract for operator-first discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompactHelpEntryPointV1 {
    /// Core built-in root routes shown to operators.
    pub built_in_routes: Vec<String>,
    /// Mounted official app root routes.
    pub mounted_app_routes: Vec<String>,
    /// Plugin namespace routes.
    pub plugin_routes: Vec<String>,
    /// Suggested next commands from the root screen.
    pub next_commands: Vec<String>,
}

/// Build compact root help with required operator routes and deterministic ordering.
pub fn build_compact_operator_help_entrypoint(
    built_in_routes: &[&str],
    mounted_app_routes: &[&str],
    plugin_routes: &[&str],
    next_commands: &[&str],
) -> Result<CompactHelpEntryPointV1, String> {
    let normalized_built_ins = unique_sorted_non_empty(built_in_routes)?;
    for required in ["dag", "config", "doctor", "plugins"] {
        if !normalized_built_ins.iter().any(|entry| entry == required) {
            return Err(format!("built_in_routes missing required root command `{required}`"));
        }
    }
    Ok(CompactHelpEntryPointV1 {
        built_in_routes: normalized_built_ins,
        mounted_app_routes: unique_sorted_non_empty(mounted_app_routes)?,
        plugin_routes: unique_sorted_non_empty(plugin_routes)?,
        next_commands: unique_sorted_non_empty(next_commands)?,
    })
}

fn unique_sorted_non_empty(values: &[&str]) -> Result<Vec<String>, String> {
    let mut normalized: Vec<String> = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err("route/help collections cannot be empty".to_string());
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::build_compact_operator_help_entrypoint;

    #[test]
    fn g001_compact_help_entrypoint_requires_core_operator_routes() {
        let report = build_compact_operator_help_entrypoint(
            &["doctor", "plugins", "dag", "config"],
            &["atlas", "genomics"],
            &["community-tools"],
            &["bijux dag --help", "bijux doctor"],
        )
        .expect("compact help should build");
        assert_eq!(report.built_in_routes, vec!["config", "dag", "doctor", "plugins"]);
        assert_eq!(report.mounted_app_routes, vec!["atlas", "genomics"]);
    }
}
