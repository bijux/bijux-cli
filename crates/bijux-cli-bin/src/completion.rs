use crate::types::ReplSession;

/// Provide command completion candidates for built-ins and plugin hooks.
#[must_use]
pub fn completion_candidates(session: &ReplSession, prefix: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    let builtins = [
        "help",
        "version",
        "doctor",
        "repl",
        "completion",
        "inspect",
        "status",
        "atlas",
        "config",
        "config get",
        "config set",
        "config unset",
        "config clear",
        "config export",
        "plugins",
        "plugins list",
        "plugins inspect",
        "plugins check",
        "plugins doctor",
        "dev",
        "dev cli",
        "dev cli routes",
        "dev cli registry",
        "dev cli env",
        "dev cli status",
        "dev cli state-doctor",
        "cli",
        "cli status",
        "cli paths",
        "history",
        "memory",
    ];
    for builtin in builtins {
        if builtin.starts_with(prefix) {
            suggestions.push(builtin.to_string());
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
