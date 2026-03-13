use super::types::ReplSession;

const CORE_BUILTIN_COMPLETIONS: &[&str] = &[
    "help",
    "version",
    "doctor",
    "repl",
    "completion",
    "inspect",
    "status",
    "config",
    "config list",
    "config get",
    "config set",
    "config unset",
    "config clear",
    "config reload",
    "config export",
    "config load",
    "plugins",
    "plugins list",
    "plugins inspect",
    "plugins check",
    "plugins doctor",
    "cli",
    "cli status",
    "cli paths",
    "history",
    "history clear",
    "memory",
    "memory list",
    "memory get",
    "memory set",
    "memory delete",
    "memory clear",
];

/// Provide command completion candidates for built-ins and plugin hooks.
#[must_use]
pub fn completion_candidates(session: &ReplSession, prefix: &str) -> Vec<String> {
    let mut suggestions = Vec::new();
    let normalized_prefix = prefix.trim_start().to_ascii_lowercase();
    let last_prefix_token = normalized_prefix.split_whitespace().last().unwrap_or_default();

    let matches_prefix = |value: &str| {
        let normalized_value = value.to_ascii_lowercase();
        if normalized_prefix.is_empty() || normalized_value.starts_with(&normalized_prefix) {
            return true;
        }

        if last_prefix_token.is_empty() {
            return false;
        }

        normalized_value
            .split_whitespace()
            .last()
            .is_some_and(|segment| segment.starts_with(last_prefix_token))
    };

    let mut push_candidate = |value: &str| {
        let candidate = value.trim();
        if candidate.is_empty() {
            return;
        }
        if matches_prefix(candidate) {
            suggestions.push(candidate.to_string());
        }
    };

    for builtin in CORE_BUILTIN_COMPLETIONS {
        push_candidate(builtin);
    }

    for values in session.completion_registries.values() {
        for value in values {
            push_candidate(value);
        }
    }

    for namespace in session.plugin_completion_hooks.keys() {
        push_candidate(namespace);
    }

    for values in session.plugin_completion_hooks.values() {
        for value in values {
            push_candidate(value);
        }
    }

    suggestions.sort();
    suggestions.dedup();
    suggestions
}

/// Register plugin completion hook for a namespace.
pub fn register_plugin_completion_hook(
    session: &mut ReplSession,
    namespace: &str,
    suggestions: Vec<String>,
) {
    let normalized_namespace = namespace.trim();
    if normalized_namespace.is_empty() {
        return;
    }

    let mut normalized = suggestions
        .into_iter()
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    session.plugin_completion_hooks.insert(normalized_namespace.to_string(), normalized);
}

/// Register extension completion registry entries under an owner key.
pub fn register_completion_registry(
    session: &mut ReplSession,
    owner: &str,
    suggestions: Vec<String>,
) {
    let normalized_owner = owner.trim();
    if normalized_owner.is_empty() {
        return;
    }

    let mut normalized = suggestions
        .into_iter()
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    session.completion_registries.insert(normalized_owner.to_string(), normalized);
}

#[cfg(test)]
mod tests {
    use super::{
        completion_candidates, register_completion_registry, register_plugin_completion_hook,
    };
    use crate::interface::repl::session::startup_repl;

    #[test]
    fn completion_matches_are_case_insensitive_and_last_token_aware() {
        let (mut session, _) = startup_repl("", None);
        register_completion_registry(
            &mut session,
            "core",
            vec!["plugins install".to_string(), "status".to_string()],
        );

        let by_case = completion_candidates(&session, "Sta");
        assert!(by_case.iter().any(|entry| entry == "status"));

        let by_last_token = completion_candidates(&session, "plugins ins");
        assert!(by_last_token.iter().any(|entry| entry == "plugins install"));
    }

    #[test]
    fn completion_registry_and_plugin_hooks_normalize_keys_and_values() {
        let (mut session, _) = startup_repl("", None);

        register_completion_registry(
            &mut session,
            " owner ",
            vec![" status ".to_string(), String::new(), "status".to_string()],
        );
        register_plugin_completion_hook(
            &mut session,
            " plugin.ns ",
            vec![" plugin.ns cmd ".to_string(), String::new(), "plugin.ns cmd".to_string()],
        );

        assert!(session.completion_registries.contains_key("owner"));
        assert_eq!(session.completion_registries["owner"], vec!["status".to_string()]);
        assert!(session.plugin_completion_hooks.contains_key("plugin.ns"));
        assert_eq!(session.plugin_completion_hooks["plugin.ns"], vec!["plugin.ns cmd".to_string()]);
    }
}
