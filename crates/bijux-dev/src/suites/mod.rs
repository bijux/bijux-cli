pub mod check;
pub mod contract;
pub mod docs;
pub mod release;
pub mod repo;
pub mod test;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteMetadata {
    pub id: &'static str,
    pub domain: &'static str,
    pub slow: bool,
    pub internal: bool,
}

pub const RELEASE_VERIFY_SUITES: &[&str] =
    &["release.validation-suite", "release.readiness", "release.compatibility-matrix"];

pub fn release_verify_suite_ids() -> Vec<&'static str> {
    release::VERIFY_FLOW.to_vec()
}

pub fn filter_suites<'a>(
    suites: &'a [SuiteMetadata],
    domain: Option<&str>,
    include_slow: bool,
    include_internal: bool,
) -> Vec<&'a SuiteMetadata> {
    suites
        .iter()
        .filter(|suite| domain.is_none_or(|d| suite.domain == d))
        .filter(|suite| include_slow || !suite.slow)
        .filter(|suite| include_internal || !suite.internal)
        .collect()
}

#[derive(Debug, Deserialize, Default)]
pub struct SuiteOverrides {
    #[serde(default)]
    pub disabled_suite_ids: Vec<String>,
}

pub fn load_suite_overrides(path: &Path) -> Result<SuiteOverrides, String> {
    if !path.exists() {
        return Ok(SuiteOverrides::default());
    }
    let payload = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&payload).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_verify_flow_is_stable() {
        assert_eq!(release_verify_suite_ids(), release::VERIFY_FLOW.to_vec());
    }

    #[test]
    fn suite_domain_collections_are_non_empty() {
        assert!(!check::IDS.is_empty());
        assert!(!test::IDS.is_empty());
        assert!(!contract::IDS.is_empty());
        assert!(!docs::IDS.is_empty());
        assert!(!repo::IDS.is_empty());
        assert!(!release::IDS.is_empty());
    }

    #[test]
    fn missing_override_file_returns_default() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("missing.json");
        let overrides = load_suite_overrides(&path).expect("load");
        assert!(overrides.disabled_suite_ids.is_empty());
    }

    #[test]
    fn filter_respects_domain_and_flags() {
        let suites = vec![
            SuiteMetadata { id: "a", domain: "contracts", slow: false, internal: false },
            SuiteMetadata { id: "b", domain: "contracts", slow: true, internal: false },
            SuiteMetadata { id: "c", domain: "repo", slow: false, internal: true },
        ];

        let selected = filter_suites(&suites, Some("contracts"), false, false);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "a");

        let selected = filter_suites(&suites, Some("contracts"), true, false);
        assert_eq!(selected.len(), 2);

        let selected = filter_suites(&suites, None, true, true);
        assert_eq!(selected.len(), 3);
    }
}
use serde::Deserialize;
use std::path::Path;
