#![forbid(unsafe_code)]
//! Canonical route model shared by catalog, registry, and dispatch policy.

pub const CLI_ROOT_ALIASES: &[&str] = &["doctor", "version", "inspect", "completion", "repl"];
pub const CLI_CONFIG_SUBCOMMANDS: &[&str] =
    &["get", "set", "unset", "clear", "reload", "export", "load", "list"];
pub const CLI_PLUGINS_SUBCOMMANDS: &[&str] = &[
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
pub const DEV_CLI_SUBCOMMANDS: &[&str] = &[
    "maintenance",
    "rustdoc",
    "release",
    "evidence",
    "config",
    "python",
    "repo",
    "dashboard",
    "quickcheck",
    "truth",
    "blockers",
    "next",
    "inventory",
    "routes",
    "registry",
    "parity",
    "docs",
    "docs-audit",
    "plugin-health",
    "status",
    "maintenance-audit",
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
pub const DEV_CLI_RUSTDOC_SUBCOMMANDS: &[&str] = &[
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
pub const DEV_CLI_RELEASE_SUBCOMMANDS: &[&str] = &[
    "status",
    "evidence",
    "readiness",
    "diff",
    "gaps",
    "summary",
    "manifest",
    "notes",
    "behavior-changes",
    "intentional-differences",
    "unresolved-gaps",
    "compatibility-leftovers",
];
pub const DEV_CLI_EVIDENCE_SUBCOMMANDS: &[&str] = &[
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
pub const DEV_CLI_CONFIG_SUBCOMMANDS: &[&str] =
    &["rust-owner", "python-owner", "ownership", "drift", "shape", "evidence-map"];
pub const DEV_CLI_PYTHON_SUBCOMMANDS: &[&str] =
    &["bridge-status", "surface-status", "sovereignty-audit", "drift", "packaging"];
pub const DEV_CLI_REPO_SUBCOMMANDS: &[&str] =
    &["health", "drift", "inventories", "generated", "stale"];
pub const DEV_CLI_MAINTENANCE_SUBCOMMANDS: &[&str] = &[
    "remaining",
    "migrated",
    "diff",
    "audit",
    "generators",
    "generate",
    "generate-all",
    "requirements",
    "flaky-tests",
    "package-metadata",
    "e2e-contract",
    "pip-audit",
    "capture-python-behavior",
    "provenance-statement",
];
pub const DEV_CLI_MAINTENANCE_STATUS_SUBCOMMANDS: &[&str] = &["inventory", "run", "run-all"];
pub const DEV_LEGACY_ALIASES: &[&str] = &[
    "inventory",
    "parity",
    "docs-audit",
    "plugin-health",
    "status",
    "maintenance-audit",
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

pub const REPL_REFERENCE_COMMANDS: &[&str] = &[
    "status",
    "doctor",
    "version",
    ":help <command>",
    ":set trace on|off",
    ":set quiet on|off",
    ":set format json|yaml|text",
    ":exit",
];

const BUILT_IN_ROUTE_PATHS: &[&str] = &[
    "status",
    "audit",
    "docs",
    "sleep",
    "atlas",
    "dev",
    "config",
    "config list",
    "history",
    "history clear",
    "memory",
    "memory list",
    "memory get",
    "memory set",
    "memory delete",
    "memory clear",
    "plugins",
    "plugins info",
    "plugins list",
    "plugins inspect",
    "plugins check",
    "plugins install",
    "plugins uninstall",
    "plugins enable",
    "plugins disable",
    "plugins scaffold",
    "plugins doctor",
    "cli status",
    "cli doctor",
    "cli version",
    "cli completion",
    "cli inspect",
    "cli repl",
    "cli paths",
    "cli self-test",
    "cli config get",
    "cli config set",
    "cli config unset",
    "cli config clear",
    "cli config reload",
    "cli config export",
    "cli config load",
    "cli config list",
    "cli plugins list",
    "cli plugins info",
    "cli plugins inspect",
    "cli plugins check",
    "cli plugins install",
    "cli plugins uninstall",
    "cli plugins enable",
    "cli plugins disable",
    "cli plugins scaffold",
    "cli plugins doctor",
    "cli plugins reserved-names",
    "cli plugins where",
    "cli plugins explain",
    "cli plugins schema",
    "dev cli inventory",
    "dev cli routes",
    "dev cli route-audit",
    "dev cli registry",
    "dev cli parity",
    "dev cli docs",
    "dev cli docs-audit",
    "dev cli maintenance",
    "dev cli rustdoc",
    "dev cli release",
    "dev cli evidence",
    "dev cli config",
    "dev cli python",
    "dev cli repo",
    "dev cli dashboard",
    "dev cli quickcheck",
    "dev cli truth",
    "dev cli blockers",
    "dev cli next",
    "dev cli plugin-health",
    "dev cli status",
    "dev cli maintenance-audit",
    "dev cli snapshots-audit",
    "dev cli fixture-audit",
    "dev cli crate-health",
    "dev cli package-health",
    "dev cli env",
    "dev cli doctor",
    "dev cli contracts",
    "dev cli runtime-identity",
    "dev cli docs-prune-plan",
    "dev cli state-audit",
    "dev cli state-doctor",
    "dev cli atlas",
    "dev cli di",
    "dev cli list-products",
    "dev cli list-plugins",
];

const ALIAS_REWRITES: &[(&str, &str)] = &[
    ("doctor", "cli doctor"),
    ("version", "cli version"),
    ("repl", "cli repl"),
    ("completion", "cli completion"),
    ("inspect", "cli inspect"),
    ("config get", "cli config get"),
    ("config set", "cli config set"),
    ("config unset", "cli config unset"),
    ("config clear", "cli config clear"),
    ("config reload", "cli config reload"),
    ("config export", "cli config export"),
    ("config load", "cli config load"),
    ("config list", "config"),
    ("plugins list", "cli plugins list"),
    ("plugins info", "plugins"),
    ("plugins inspect", "cli plugins inspect"),
    ("plugins check", "cli plugins check"),
    ("plugins install", "cli plugins install"),
    ("plugins uninstall", "cli plugins uninstall"),
    ("plugins enable", "cli plugins enable"),
    ("plugins disable", "cli plugins disable"),
    ("plugins scaffold", "cli plugins scaffold"),
    ("plugins doctor", "cli plugins doctor"),
    ("plugins reserved-names", "cli plugins reserved-names"),
    ("plugins where", "cli plugins where"),
    ("plugins explain", "cli plugins explain"),
    ("plugins schema", "cli plugins schema"),
    ("dev inventory", "dev cli inventory"),
    ("dev route-audit", "dev cli route-audit"),
    ("dev parity", "dev cli parity"),
    ("dev docs-audit", "dev cli docs-audit"),
    ("dev plugin-health", "dev cli plugin-health"),
    ("dev status", "dev cli status"),
    ("dev package-health", "dev cli package-health"),
    ("dev doctor", "dev cli doctor"),
    ("dev runtime-identity", "dev cli runtime-identity"),
    ("dev state-audit", "dev cli state-audit"),
    ("dev state-doctor", "dev cli state-doctor"),
    ("dev list-plugins", "dev cli list-plugins"),
];

fn contains(values: &[&str], value: &str) -> bool {
    values.contains(&value)
}

pub fn built_in_route_paths() -> &'static [&'static str] {
    BUILT_IN_ROUTE_PATHS
}

pub fn alias_rewrites() -> &'static [(&'static str, &'static str)] {
    ALIAS_REWRITES
}

pub fn is_dev_legacy_alias(value: &str) -> bool {
    contains(DEV_LEGACY_ALIASES, value)
}

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
                && c == "maintenance"
                && contains(DEV_CLI_MAINTENANCE_SUBCOMMANDS, d) =>
        {
            true
        }
        [a, b, c, d, e]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && d == "status"
                && contains(DEV_CLI_MAINTENANCE_STATUS_SUBCOMMANDS, e) =>
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
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "python"
                && contains(DEV_CLI_PYTHON_SUBCOMMANDS, d) =>
        {
            true
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "repo" && contains(DEV_CLI_REPO_SUBCOMMANDS, d) =>
        {
            true
        }
        [a, b, c] if a == "cli" && b == "hold" && c == "interruptible" => true,
        _ => false,
    }
}
