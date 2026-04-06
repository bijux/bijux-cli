#![forbid(unsafe_code)]
//! Runtime version resolution for display and compatibility checks.

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_SEMVER_VERSION: Option<&str> = option_env!("BIJUX_BUILD_SEMVER_VERSION");
const BUILD_DISPLAY_VERSION: Option<&str> = option_env!("BIJUX_BUILD_DISPLAY_VERSION");
const BUILD_VERSION_SOURCE: Option<&str> = option_env!("BIJUX_BUILD_VERSION_SOURCE");
const BUILD_GIT_COMMIT: Option<&str> = option_env!("BIJUX_BUILD_GIT_COMMIT");
const BUILD_GIT_DIRTY: Option<&str> = option_env!("BIJUX_BUILD_GIT_DIRTY");

const VERSION_SOURCE_OVERRIDE: &str = "override";
const VERSION_SOURCE_GIT_TAG: &str = "git-tag";
const VERSION_SOURCE_GIT_TAG_DERIVED: &str = "git-tag-derived";
const VERSION_SOURCE_PACKAGE_FALLBACK: &str = "package-fallback";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeVersionInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub semver: &'static str,
    pub source: &'static str,
    pub git_commit: Option<&'static str>,
    pub git_dirty: Option<bool>,
    pub build_profile: &'static str,
}

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

pub(crate) fn runtime_version_source() -> &'static str {
    match BUILD_VERSION_SOURCE {
        Some(VERSION_SOURCE_OVERRIDE) => VERSION_SOURCE_OVERRIDE,
        Some(VERSION_SOURCE_GIT_TAG) => VERSION_SOURCE_GIT_TAG,
        Some(VERSION_SOURCE_GIT_TAG_DERIVED) => VERSION_SOURCE_GIT_TAG_DERIVED,
        Some(VERSION_SOURCE_PACKAGE_FALLBACK) => VERSION_SOURCE_PACKAGE_FALLBACK,
        Some(_) | None => VERSION_SOURCE_PACKAGE_FALLBACK,
    }
}

pub(crate) fn runtime_git_commit() -> Option<&'static str> {
    BUILD_GIT_COMMIT
}

pub(crate) fn runtime_git_dirty() -> Option<bool> {
    match BUILD_GIT_DIRTY {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    }
}

pub(crate) fn runtime_build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

pub(crate) fn runtime_version_info() -> RuntimeVersionInfo {
    RuntimeVersionInfo {
        name: "bijux",
        version: runtime_version(),
        semver: runtime_semver(),
        source: runtime_version_source(),
        git_commit: runtime_git_commit(),
        git_dirty: runtime_git_dirty(),
        build_profile: runtime_build_profile(),
    }
}

pub(crate) fn runtime_version_line() -> String {
    let info = runtime_version_info();
    let mut line = format!("{} version {} ({})", info.name, info.version, info.source);
    if info.source != VERSION_SOURCE_GIT_TAG_DERIVED {
        if let Some(commit) = info.git_commit {
            line.push_str(", build ");
            line.push_str(commit);
            if info.git_dirty == Some(true) {
                line.push_str("-dirty");
            }
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::{
        runtime_build_profile, runtime_git_dirty, runtime_semver, runtime_version,
        runtime_version_info, runtime_version_line, runtime_version_source,
    };

    #[test]
    fn runtime_semver_is_valid() {
        assert!(Version::parse(runtime_semver()).is_ok());
    }

    #[test]
    fn runtime_version_is_non_empty() {
        assert!(!runtime_version().trim().is_empty());
    }

    #[test]
    fn runtime_version_source_is_known_value() {
        assert!(matches!(
            runtime_version_source(),
            "override" | "git-tag" | "git-tag-derived" | "package-fallback"
        ));
    }

    #[test]
    fn runtime_version_uses_tag_style_prefix() {
        assert!(runtime_version().starts_with('v'));
    }

    #[test]
    fn runtime_git_dirty_flag_parses_known_values() {
        assert!(runtime_git_dirty().is_none_or(|value| matches!(value, true | false)));
    }

    #[test]
    fn runtime_version_info_is_self_consistent() {
        let info = runtime_version_info();
        assert_eq!(info.version, runtime_version());
        assert_eq!(info.semver, runtime_semver());
        assert_eq!(info.source, runtime_version_source());
        assert_eq!(info.build_profile, runtime_build_profile());
    }

    #[test]
    fn runtime_version_line_is_docker_style_prefix() {
        let line = runtime_version_line();
        assert!(line.starts_with("bijux version "));
        assert!(!line.trim().is_empty());
    }
}
