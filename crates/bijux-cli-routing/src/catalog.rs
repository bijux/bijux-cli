#![forbid(unsafe_code)]
//! Canonical command catalog for normalization and route recognition.

const CLI_ROOT_ALIASES: &[&str] = &["doctor", "version", "inspect", "completion", "repl"];
const CLI_CONFIG_SUBCOMMANDS: &[&str] =
    &["get", "set", "unset", "clear", "reload", "export", "load", "list"];
const CLI_PLUGINS_SUBCOMMANDS: &[&str] = &[
    "list",
    "info",
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
    "scripts",
    "rustdoc",
    "release",
    "evidence",
    "config",
    "inventory",
    "routes",
    "registry",
    "parity",
    "docs",
    "docs-audit",
    "plugin-health",
    "status",
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
    "atlas",
    "di",
    "list-products",
    "list-plugins",
];
const DEV_CLI_RUSTDOC_SUBCOMMANDS: &[&str] = &[
    "audit",
    "coverage",
    "broken-links",
    "public-api",
    "examples",
    "migrate-website-api-docs",
    "build-proof",
    "workspace-coverage-proof",
    "python-link-proof",
];
const DEV_CLI_RELEASE_SUBCOMMANDS: &[&str] = &[
    "status",
    "evidence",
    "readiness",
    "diff",
    "gaps",
    "changelog-burden",
    "migrate-changelog",
    "summary",
    "manifest",
    "notes",
    "behavior-changes",
    "intentional-differences",
    "unresolved-gaps",
    "compatibility-leftovers",
];
const DEV_CLI_EVIDENCE_SUBCOMMANDS: &[&str] = &[
    "list",
    "show",
    "audit",
    "stale",
    "matrix",
    "website-export",
    "ci-export",
    "release-export",
    "command-map",
    "parity-map",
];
const DEV_CLI_CONFIG_SUBCOMMANDS: &[&str] =
    &["rust-owner", "python-owner", "ownership", "drift", "shape", "evidence-map"];
const DEV_CLI_SCRIPTS_SUBCOMMANDS: &[&str] = &[
    "remaining",
    "migrated",
    "diff",
    "audit",
    "package-metadata",
    "e2e-contract",
    "pip-audit",
    "capture-python-behavior",
    "provenance-statement",
];
const DEV_LEGACY_ALIASES: &[&str] = &[
    "inventory",
    "parity",
    "docs-audit",
    "plugin-health",
    "status",
    "script-audit",
    "crate-health",
    "package-health",
    "route-audit",
    "doctor",
    "runtime-identity",
    "docs-prune-plan",
    "state-audit",
    "state-doctor",
    "atlas",
    "di",
    "list-products",
    "list-plugins",
];

fn contains(values: &[&str], value: &str) -> bool {
    values.contains(&value)
}

/// Canonical CLI root aliases supported by route normalization.
#[must_use]
pub fn cli_root_aliases() -> &'static [&'static str] {
    CLI_ROOT_ALIASES
}

/// Canonical `cli config` subcommands supported by route normalization.
#[must_use]
pub fn cli_config_subcommands() -> &'static [&'static str] {
    CLI_CONFIG_SUBCOMMANDS
}

/// Canonical `cli plugins` subcommands supported by route normalization.
#[must_use]
pub fn cli_plugins_subcommands() -> &'static [&'static str] {
    CLI_PLUGINS_SUBCOMMANDS
}

/// Canonical `dev cli` subcommands supported by route normalization.
#[must_use]
pub fn dev_cli_subcommands() -> &'static [&'static str] {
    DEV_CLI_SUBCOMMANDS
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
                && (contains(CLI_ROOT_ALIASES, b)
                    || b == "status"
                    || b == "paths"
                    || b == "self-test") =>
        {
            true
        }
        [a, b, c] if a == "cli" && b == "config" && contains(CLI_CONFIG_SUBCOMMANDS, c) => true,
        [a] if a == "config" || a == "history" || a == "memory" || a == "plugins" => true,
        [a, b] if a == "history" && b == "clear" => true,
        [a, b]
            if a == "memory"
                && (b == "list" || b == "get" || b == "set" || b == "delete" || b == "clear") =>
        {
            true
        }
        [a] if a == "status" || a == "audit" || a == "docs" || a == "sleep" || a == "atlas" => true,
        [a, b, c] if a == "cli" && b == "plugins" && contains(CLI_PLUGINS_SUBCOMMANDS, c) => true,
        [a, b, c] if a == "dev" && b == "cli" && contains(DEV_CLI_SUBCOMMANDS, c) => true,
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "scripts"
                && contains(DEV_CLI_SCRIPTS_SUBCOMMANDS, d) =>
        {
            true
        }
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "rustdoc"
                && contains(DEV_CLI_RUSTDOC_SUBCOMMANDS, d) =>
        {
            true
        }
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "release"
                && contains(DEV_CLI_RELEASE_SUBCOMMANDS, d) =>
        {
            true
        }
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "evidence"
                && contains(DEV_CLI_EVIDENCE_SUBCOMMANDS, d) =>
        {
            true
        }
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "config"
                && contains(DEV_CLI_CONFIG_SUBCOMMANDS, d) =>
        {
            true
        }
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
