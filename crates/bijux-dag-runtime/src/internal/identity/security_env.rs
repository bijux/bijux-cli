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
