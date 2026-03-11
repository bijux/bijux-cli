#![forbid(unsafe_code)]
//! Canonical route model shared by catalog, registry, and dispatch policy.

use std::collections::BTreeSet;
use std::sync::OnceLock;

pub const CLI_ROOT_ALIASES: &[&str] = &["doctor", "version", "inspect", "completion", "repl"];
pub const CLI_CONFIG_SUBCOMMANDS: &[&str] = &[
    "get", "set", "unset", "clear", "reload", "export", "load", "list",
];
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
pub const DEV_CLI_CONFIG_SUBCOMMANDS: &[&str] = &[
    "rust-owner",
    "python-owner",
    "ownership",
    "drift",
    "shape",
    "evidence-map",
];
pub const DEV_CLI_PYTHON_SUBCOMMANDS: &[&str] = &[
    "bridge-status",
    "surface-status",
    "sovereignty-audit",
    "drift",
    "packaging",
];
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

static BUILT_IN_ROUTE_PATHS: OnceLock<Vec<String>> = OnceLock::new();
static KNOWN_ROUTE_PATHS: OnceLock<BTreeSet<String>> = OnceLock::new();

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

fn push_prefixed_routes(routes: &mut Vec<String>, prefix: &str, leaves: &[&str]) {
    routes.extend(leaves.iter().map(|leaf| format!("{prefix} {leaf}")));
}

fn push_prefixed_routes_to_set(routes: &mut BTreeSet<String>, prefix: &str, leaves: &[&str]) {
    for leaf in leaves {
        routes.insert(format!("{prefix} {leaf}"));
    }
}

fn build_built_in_route_paths() -> Vec<String> {
    let mut routes = vec![
        "status".to_string(),
        "audit".to_string(),
        "docs".to_string(),
        "sleep".to_string(),
        "atlas".to_string(),
        "dev".to_string(),
        "config".to_string(),
        "config list".to_string(),
        "history".to_string(),
        "history clear".to_string(),
        "memory".to_string(),
        "memory list".to_string(),
        "memory get".to_string(),
        "memory set".to_string(),
        "memory delete".to_string(),
        "memory clear".to_string(),
        "plugins".to_string(),
        "plugins info".to_string(),
        "plugins list".to_string(),
        "plugins inspect".to_string(),
        "plugins check".to_string(),
        "plugins install".to_string(),
        "plugins uninstall".to_string(),
        "plugins enable".to_string(),
        "plugins disable".to_string(),
        "plugins scaffold".to_string(),
        "plugins doctor".to_string(),
        "cli status".to_string(),
        "cli paths".to_string(),
        "cli self-test".to_string(),
    ];

    push_prefixed_routes(&mut routes, "cli", CLI_ROOT_ALIASES);
    push_prefixed_routes(&mut routes, "cli config", CLI_CONFIG_SUBCOMMANDS);
    push_prefixed_routes(&mut routes, "cli plugins", CLI_PLUGINS_SUBCOMMANDS);
    push_prefixed_routes(&mut routes, "dev cli", DEV_CLI_SUBCOMMANDS);

    routes.sort();
    routes.dedup();
    routes
}

fn build_known_route_paths() -> BTreeSet<String> {
    let mut routes: BTreeSet<String> = built_in_route_paths().iter().cloned().collect();
    push_prefixed_routes_to_set(
        &mut routes,
        "dev cli maintenance",
        DEV_CLI_MAINTENANCE_SUBCOMMANDS,
    );
    push_prefixed_routes_to_set(
        &mut routes,
        "dev cli maintenance status",
        DEV_CLI_MAINTENANCE_STATUS_SUBCOMMANDS,
    );
    push_prefixed_routes_to_set(&mut routes, "dev cli rustdoc", DEV_CLI_RUSTDOC_SUBCOMMANDS);
    push_prefixed_routes_to_set(&mut routes, "dev cli release", DEV_CLI_RELEASE_SUBCOMMANDS);
    push_prefixed_routes_to_set(
        &mut routes,
        "dev cli evidence",
        DEV_CLI_EVIDENCE_SUBCOMMANDS,
    );
    push_prefixed_routes_to_set(&mut routes, "dev cli config", DEV_CLI_CONFIG_SUBCOMMANDS);
    push_prefixed_routes_to_set(&mut routes, "dev cli python", DEV_CLI_PYTHON_SUBCOMMANDS);
    push_prefixed_routes_to_set(&mut routes, "dev cli repo", DEV_CLI_REPO_SUBCOMMANDS);
    routes
}

pub fn built_in_route_paths() -> &'static [String] {
    BUILT_IN_ROUTE_PATHS.get_or_init(build_built_in_route_paths)
}

pub fn alias_rewrites() -> &'static [(&'static str, &'static str)] {
    ALIAS_REWRITES
}

pub fn is_dev_legacy_alias(value: &str) -> bool {
    contains(DEV_LEGACY_ALIASES, value)
}

pub fn is_known_route(path: &[String]) -> bool {
    if path.is_empty() {
        return false;
    }

    let key = path.join(" ");
    let canonical = ALIAS_REWRITES
        .iter()
        .find(|(alias, _)| *alias == key.as_str())
        .map(|(_, canonical)| *canonical)
        .unwrap_or(key.as_str());

    KNOWN_ROUTE_PATHS
        .get_or_init(build_known_route_paths)
        .contains(canonical)
}
