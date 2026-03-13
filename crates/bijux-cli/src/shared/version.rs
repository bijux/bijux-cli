#![forbid(unsafe_code)]
//! Runtime version resolution for display and compatibility checks.

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_SEMVER_VERSION: Option<&str> = option_env!("BIJUX_BUILD_SEMVER_VERSION");
const BUILD_DISPLAY_VERSION: Option<&str> = option_env!("BIJUX_BUILD_DISPLAY_VERSION");

pub(crate) const fn runtime_semver() -> &'static str {
    match BUILD_SEMVER_VERSION {
        Some(version) => version,
        None => PACKAGE_VERSION,
    }
}

pub(crate) const fn runtime_version() -> &'static str {
    match BUILD_DISPLAY_VERSION {
        Some(version) => version,
        None => runtime_semver(),
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::{runtime_semver, runtime_version};

    #[test]
    fn runtime_semver_is_valid() {
        assert!(Version::parse(runtime_semver()).is_ok());
    }

    #[test]
    fn runtime_version_is_non_empty() {
        assert!(!runtime_version().trim().is_empty());
    }
}
