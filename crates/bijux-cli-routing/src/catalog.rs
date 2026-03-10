#![forbid(unsafe_code)]
//! Canonical command catalog for normalization and route recognition.

const CLI_ROOT_ALIASES: &[&str] = &["doctor", "version", "inspect", "completion", "repl"];
const CLI_CONFIG_SUBCOMMANDS: &[&str] = &["get", "set", "unset", "clear", "reload", "export", "load"];
const CLI_PLUGINS_SUBCOMMANDS: &[&str] = &[
    "list",
    "inspect",
    "check",
    "install",
    "uninstall",
    "enable",
    "disable",
    "scaffold",
    "doctor",
    "reserved-names",
    "where",
    "explain",
    "schema",
];
const DEV_CLI_SUBCOMMANDS: &[&str] = &[
    "inventory",
    "routes",
    "registry",
    "parity",
    "docs",
    "docs-audit",
    "plugin-health",
    "status",
    "scripts-audit",
    "script-audit",
    "snapshots-audit",
    "fixture-audit",
    "crate-health",
    "package-health",
    "route-audit",
    "env",
    "doctor",
    "contracts",
    "runtime-identity",
    "docs-prune-plan",
    "state-audit",
    "state-doctor",
];
const DEV_LEGACY_ALIASES: &[&str] = &[
    "inventory",
    "routes",
    "registry",
    "parity",
    "docs-audit",
    "plugin-health",
    "status",
    "scripts-audit",
    "script-audit",
    "crate-health",
    "package-health",
    "route-audit",
    "doctor",
    "runtime-identity",
    "docs-prune-plan",
    "state-audit",
    "state-doctor",
];

fn contains(values: &[&str], value: &str) -> bool {
    values.contains(&value)
}

/// Normalize path aliases into canonical command paths.
#[must_use]
pub fn normalize_command_path(path: &[String]) -> Vec<String> {
    match path {
        [a] if contains(CLI_ROOT_ALIASES, a) => vec!["cli".to_string(), a.clone()],
        [a, b] if a == "config" && contains(CLI_CONFIG_SUBCOMMANDS, b) => {
            vec!["cli".to_string(), "config".to_string(), b.clone()]
        }
        [a, b] if a == "plugins" && contains(CLI_PLUGINS_SUBCOMMANDS, b) => {
            vec!["cli".to_string(), "plugins".to_string(), b.clone()]
        }
        [a, b] if a == "dev" && contains(DEV_LEGACY_ALIASES, b) => {
            vec!["dev".to_string(), "cli".to_string(), b.clone()]
        }
        _ => path.to_vec(),
    }
}

/// Return true when normalized path is a known built-in command route.
#[must_use]
pub fn is_known_route(path: &[String]) -> bool {
    match path {
        [a, b]
            if a == "cli"
                && (contains(CLI_ROOT_ALIASES, b) || b == "status" || b == "paths" || b == "self-test") =>
        {
            true
        }
        [a, b, c] if a == "cli" && b == "config" && contains(CLI_CONFIG_SUBCOMMANDS, c) => true,
        [a] if a == "config" || a == "history" || a == "memory" => true,
        [a, b] if a == "memory" && b == "list" => true,
        [a] if a == "status" || a == "audit" || a == "docs" || a == "sleep" => true,
        [a, b, c] if a == "cli" && b == "plugins" && contains(CLI_PLUGINS_SUBCOMMANDS, c) => {
            true
        }
        [a, b, c] if a == "dev" && b == "cli" && contains(DEV_CLI_SUBCOMMANDS, c) => true,
        [a, b, c] if a == "cli" && b == "hold" && c == "interruptible" => true,
        _ => false,
    }
}

/// REPL reference commands rendered in command help.
#[must_use]
pub fn repl_reference_commands() -> &'static [&'static str] {
    &[
        "status",
        "doctor",
        "version",
        ":help <command>",
        ":set trace on|off",
        ":set quiet on|off",
        ":set format json|yaml|text",
        ":exit",
    ]
}
