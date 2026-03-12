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

    for builtin in CORE_BUILTIN_COMPLETIONS {
        if builtin.starts_with(prefix) {
            suggestions.push(builtin.to_string());
        }
    }

    for values in session.completion_registries.values() {
        for value in values {
            if value.starts_with(prefix) {
                suggestions.push(value.clone());
            }
        }
    }

    for namespace in session.plugin_completion_hooks.keys() {
        if namespace.starts_with(prefix) {
            suggestions.push(namespace.clone());
        }
    }

    for values in session.plugin_completion_hooks.values() {
        for value in values {
            if value.starts_with(prefix) {
                suggestions.push(value.clone());
            }
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
    session.plugin_completion_hooks.insert(namespace.to_string(), suggestions);
}

/// Register extension completion registry entries under an owner key.
pub fn register_completion_registry(
    session: &mut ReplSession,
    owner: &str,
    suggestions: Vec<String>,
) {
    session.completion_registries.insert(owner.to_string(), suggestions);
}
