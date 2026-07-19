use bijux_dag_core::{env_allowlist_pattern_is_exact, Graph, Node};
use std::collections::BTreeMap;

fn matches_pattern(key: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        return key.starts_with(prefix);
    }
    key == pattern
}

pub fn is_allowed_env_key(key: &str, allowlist: &[String]) -> bool {
    allowlist.iter().any(|pattern| matches_pattern(key, pattern))
}

pub fn is_denied_env_key(key: &str, denylist: &[String]) -> bool {
    denylist.iter().any(|pattern| matches_pattern(key, pattern))
}

pub fn shape_environment(
    ambient: &BTreeMap<String, String>,
    clean_env: bool,
    allowlist: &[String],
    denylist: &[String],
    explicit: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut shaped = BTreeMap::new();
    if !clean_env {
        for (key, value) in ambient {
            if is_denied_env_key(key, denylist) {
                continue;
            }
            if !allowlist.is_empty() && !is_allowed_env_key(key, allowlist) {
                continue;
            }
            shaped.insert(key.clone(), value.clone());
        }
    }
    for (key, value) in explicit {
        if is_denied_env_key(key, denylist) {
            continue;
        }
        if !allowlist.is_empty() && !is_allowed_env_key(key, allowlist) {
            continue;
        }
        shaped.insert(key.clone(), value.clone());
    }
    shaped
}

pub fn effective_env_allowlist(node: &Node) -> Vec<String> {
    let mut allowlist = node.env_allowlist.clone();
    if let Some(container) = &node.container {
        allowlist.extend(container.env_allowlist.iter().cloned());
    }
    allowlist.sort();
    allowlist.dedup();
    allowlist
}

pub fn declared_environment(
    ambient: &BTreeMap<String, String>,
    clean_env: bool,
    allowlist: &[String],
    denylist: &[String],
) -> BTreeMap<String, String> {
    if allowlist.is_empty() {
        return BTreeMap::new();
    }
    let explicit = if clean_env {
        ambient
            .iter()
            .filter(|(key, _)| is_allowed_env_key(key, allowlist))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    } else {
        BTreeMap::new()
    };
    shape_environment(ambient, clean_env, allowlist, denylist, &explicit)
}

pub fn missing_required_env_keys(
    ambient: &BTreeMap<String, String>,
    allowlist: &[String],
) -> Vec<String> {
    let mut missing = allowlist
        .iter()
        .filter(|pattern| {
            env_allowlist_pattern_is_exact(pattern) && !ambient.contains_key(*pattern)
        })
        .cloned()
        .collect::<Vec<_>>();
    missing.sort();
    missing.dedup();
    missing
}

pub(crate) fn validate_graph_environment_bindings(
    graph: &Graph,
    ambient: &BTreeMap<String, String>,
) -> Result<(), String> {
    let mut failures = Vec::new();
    for node in &graph.nodes {
        let allowlist = effective_env_allowlist(node);
        let missing = missing_required_env_keys(ambient, &allowlist);
        if !missing.is_empty() {
            failures.push(format!(
                "node '{}' is missing required environment bindings: {}",
                node.id,
                missing.join(", ")
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}
