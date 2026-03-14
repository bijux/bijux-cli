#![forbid(unsafe_code)]
//! Canonical route model shared by catalog, registry, and dispatch policy.

use std::collections::BTreeSet;
use std::sync::OnceLock;

pub const CLI_ROOT_ALIASES: &[&str] = &["doctor", "version", "completion", "repl", "inspect"];
pub const ROOT_RUNTIME_COMMANDS: &[&str] =
    &["status", "audit", "docs", "sleep", "doctor", "version", "install"];
pub const ROOT_STATE_COMMANDS: &[&str] = &["history", "memory"];
pub const ROOT_INTERACTION_COMMANDS: &[&str] = &["repl", "completion", "cli"];
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
    ("cli config", "config"),
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
];

fn push_prefixed_routes(routes: &mut Vec<String>, prefix: &str, leaves: &[&str]) {
    routes.extend(leaves.iter().map(|leaf| format!("{prefix} {leaf}")));
}

fn build_built_in_route_paths() -> Vec<String> {
    let mut routes = vec![
        "status".to_string(),
        "audit".to_string(),
        "docs".to_string(),
        "sleep".to_string(),
        "install".to_string(),
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
    routes.sort();
    routes.dedup();
    routes
}

fn build_known_route_paths() -> BTreeSet<String> {
    built_in_route_paths().iter().cloned().collect()
}

pub fn built_in_route_paths() -> &'static [String] {
    BUILT_IN_ROUTE_PATHS.get_or_init(build_built_in_route_paths)
}

pub fn alias_rewrites() -> &'static [(&'static str, &'static str)] {
    ALIAS_REWRITES
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

    KNOWN_ROUTE_PATHS.get_or_init(build_known_route_paths).contains(canonical)
}
