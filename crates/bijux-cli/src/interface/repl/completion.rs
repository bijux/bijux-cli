use std::sync::OnceLock;

use super::types::ReplSession;
use super::types::{
    REPL_COMPLETION_ENTRY_MAX_CHARS, REPL_COMPLETION_MAX_CANDIDATES,
    REPL_COMPLETION_REGISTRY_MAX_ENTRIES, REPL_COMPLETION_REGISTRY_MAX_OWNERS,
    REPL_PLUGIN_COMPLETION_MAX_NAMESPACES,
};
use crate::routing::model::built_in_route_paths;

const STATIC_REPL_COMPLETIONS: &[&str] = &[
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
    ":help",
    ":set",
    ":set trace",
    ":set quiet",
    ":set format",
    ":exit",
    ":quit",
];
static CORE_COMPLETION_CACHE: OnceLock<Vec<String>> = OnceLock::new();

fn normalize_completion_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut normalized = trimmed
        .chars()
        .filter(|ch| !ch.is_control())
        .take(REPL_COMPLETION_ENTRY_MAX_CHARS)
        .collect::<String>();
    normalized = normalized.trim().to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn core_completion_candidates() -> &'static [String] {
    CORE_COMPLETION_CACHE.get_or_init(|| {
        let mut values = built_in_route_paths().to_vec();
        values.extend(STATIC_REPL_COMPLETIONS.iter().map(|entry| entry.to_string()));
        let mut normalized = values
            .into_iter()
            .filter_map(|entry| normalize_completion_value(&entry))
            .collect::<Vec<_>>();
        normalized.sort();
        normalized.dedup();
        normalized
    })
}

/// Provide command completion candidates for built-ins and plugin hooks.
#[must_use]
pub fn completion_candidates(session: &ReplSession, prefix: &str) -> Vec<String> {
    let mut suggestions = std::collections::BTreeSet::new();
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
        let Some(candidate) = normalize_completion_value(value) else {
            return;
        };
        if matches_prefix(&candidate) {
            suggestions.insert(candidate);
            while suggestions.len() > REPL_COMPLETION_MAX_CANDIDATES {
                let _ = suggestions.pop_last();
            }
        }
    };

    for builtin in core_completion_candidates() {
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

    suggestions.into_iter().collect()
}

/// Register plugin completion hook for a namespace.
pub fn register_plugin_completion_hook(
    session: &mut ReplSession,
    namespace: &str,
    suggestions: Vec<String>,
) {
    let Some(normalized_namespace) = normalize_completion_value(namespace) else {
        return;
    };

    let mut normalized = suggestions
        .into_iter()
        .filter_map(|entry| normalize_completion_value(&entry))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized.truncate(REPL_COMPLETION_REGISTRY_MAX_ENTRIES);

    session.plugin_completion_hooks.insert(normalized_namespace, normalized);
    while session.plugin_completion_hooks.len() > REPL_PLUGIN_COMPLETION_MAX_NAMESPACES {
        let _ = session.plugin_completion_hooks.pop_last();
    }
}

/// Register extension completion registry entries under an owner key.
pub fn register_completion_registry(
    session: &mut ReplSession,
    owner: &str,
    suggestions: Vec<String>,
) {
    let Some(normalized_owner) = normalize_completion_value(owner) else {
        return;
    };

    let mut normalized = suggestions
        .into_iter()
        .filter_map(|entry| normalize_completion_value(&entry))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized.truncate(REPL_COMPLETION_REGISTRY_MAX_ENTRIES);

    session.completion_registries.insert(normalized_owner, normalized);
    while session.completion_registries.len() > REPL_COMPLETION_REGISTRY_MAX_OWNERS {
        let _ = session.completion_registries.pop_last();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        completion_candidates, register_completion_registry, register_plugin_completion_hook,
    };
    use crate::interface::repl::session::startup_repl;
    use crate::interface::repl::types::{
        REPL_COMPLETION_ENTRY_MAX_CHARS, REPL_COMPLETION_MAX_CANDIDATES,
        REPL_COMPLETION_REGISTRY_MAX_ENTRIES, REPL_COMPLETION_REGISTRY_MAX_OWNERS,
        REPL_PLUGIN_COMPLETION_MAX_NAMESPACES,
    };

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

    #[test]
    fn completion_registry_caps_entry_count_and_entry_size() {
        let (mut session, _) = startup_repl("", None);
        let oversized = "x".repeat(REPL_COMPLETION_ENTRY_MAX_CHARS + 64);
        let values = (0..(REPL_COMPLETION_REGISTRY_MAX_ENTRIES + 20))
            .map(|idx| format!("item-{idx:04}-{oversized}"))
            .collect::<Vec<_>>();
        register_completion_registry(&mut session, "owner", values);
        assert_eq!(
            session.completion_registries["owner"].len(),
            REPL_COMPLETION_REGISTRY_MAX_ENTRIES
        );
        assert!(session.completion_registries["owner"]
            .iter()
            .all(|entry| entry.chars().count() <= REPL_COMPLETION_ENTRY_MAX_CHARS));
    }

    #[test]
    fn completion_candidates_are_bounded() {
        let (mut session, _) = startup_repl("", None);
        let values = (0..(REPL_COMPLETION_MAX_CANDIDATES + 32))
            .map(|idx| format!("status item-{idx:04}"))
            .collect::<Vec<_>>();
        register_completion_registry(&mut session, "owner", values);
        let candidates = completion_candidates(&session, "status item-");
        assert_eq!(candidates.len(), REPL_COMPLETION_MAX_CANDIDATES);
        assert_eq!(candidates.first().map(String::as_str), Some("status item-0000"));
        assert_eq!(candidates.last().map(String::as_str), Some("status item-0511"));
    }

    #[test]
    fn completion_registry_owner_count_is_bounded() {
        let (mut session, _) = startup_repl("", None);
        for idx in 0..(REPL_COMPLETION_REGISTRY_MAX_OWNERS + 20) {
            register_completion_registry(
                &mut session,
                &format!("owner-{idx:04}"),
                vec!["status".to_string()],
            );
        }
        assert_eq!(session.completion_registries.len(), REPL_COMPLETION_REGISTRY_MAX_OWNERS);
    }

    #[test]
    fn plugin_completion_namespace_count_is_bounded() {
        let (mut session, _) = startup_repl("", None);
        for idx in 0..(REPL_PLUGIN_COMPLETION_MAX_NAMESPACES + 20) {
            register_plugin_completion_hook(
                &mut session,
                &format!("plugin-{idx:04}"),
                vec!["status".to_string()],
            );
        }
        assert_eq!(session.plugin_completion_hooks.len(), REPL_PLUGIN_COMPLETION_MAX_NAMESPACES);
    }
}
