//! Shared status-script ID derivation for `scripts/status/*.py`.

use std::path::Path;

/// Returns the status script kind prefix (`generate|enforce|check|warn|run`).
#[must_use]
pub fn status_script_kind(script_path: &str) -> Option<&'static str> {
    let file = script_path.rsplit('/').next().unwrap_or(script_path);
    if file.starts_with("generate_") {
        return Some("generate");
    }
    if file.starts_with("enforce_") {
        return Some("enforce");
    }
    if file.starts_with("check_") {
        return Some("check");
    }
    if file.starts_with("warn_") {
        return Some("warn");
    }
    if file.starts_with("run_") {
        return Some("run");
    }
    None
}

/// Returns the stable status-script slug from a `scripts/status/*.py` path.
#[must_use]
pub fn status_script_slug(script_path: &str) -> Option<String> {
    let kind = status_script_kind(script_path)?;
    let file = script_path.rsplit('/').next().unwrap_or(script_path);
    let stem = file.strip_suffix(".py").unwrap_or(file);
    let stem = stem.strip_prefix(&format!("{kind}_")).unwrap_or(stem);
    Some(
        stem.chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_uppercase() } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-"),
    )
}

/// Returns canonical `STATUS-SCRIPT-<KIND>-<SLUG>` ID for `scripts/status/*.py`.
#[must_use]
pub fn status_script_id(script_path: &str) -> Option<String> {
    let is_py = Path::new(script_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("py"));
    if !script_path.starts_with("scripts/status/") || !is_py {
        return None;
    }
    let kind = status_script_kind(script_path)?;
    let slug = status_script_slug(script_path)?;
    Some(format!("STATUS-SCRIPT-{}-{slug}", kind.to_ascii_uppercase()))
}

#[cfg(test)]
mod tests {
    use super::{status_script_id, status_script_kind};

    #[test]
    fn status_script_id_is_stable_for_generate_reports() {
        let path = "scripts/status/generate_state_audit_reports.py";
        assert_eq!(status_script_kind(path), Some("generate"));
        assert_eq!(
            status_script_id(path).as_deref(),
            Some("STATUS-SCRIPT-GENERATE-STATE-AUDIT-REPORTS")
        );
    }
}
