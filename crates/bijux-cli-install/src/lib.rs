#![forbid(unsafe_code)]
//! Installation and distribution surfaces.

mod compatibility;
mod completion;
mod diagnostics;
mod metadata;
mod paths;
mod state;

pub use compatibility::{
    default_compatibility_paths, discover_compatibility_paths, load_compatibility_config,
    parse_compatibility_config, write_compatibility_config, CompatibilityConfig,
    CompatibilityError, CompatibilityPaths, PathOverrides, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
    ENV_PLUGINS_PATH,
};
pub use completion::{
    cargo_compatibility_note, completion_script, pip_compatibility_note, post_install_hint,
    CompletionShell,
};
pub use diagnostics::{install_health_report, InstallHealthReport};
pub use metadata::{
    canonical_crate_name, cargo_install_strategy, has_secondary_executable_conflict,
    installer_marker, pip_install_strategy, publish_compatibility_package_alias,
    Ecosystem, InstallStrategy, PackageChannel, CANONICAL_EXECUTABLE,
};
pub use paths::{
    detect_stale_wrapper_scripts, discover_path_binaries, initialize_first_run_state,
    legacy_installer_conflicts, resolve_active_binary,
};
pub use state::{
    acquire_state_lock, ensure_history_file, ensure_plugins_dir, run_config_migrations,
    StateLockGuard,
};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn cargo_channels_resolve_to_same_canonical_executable() {
        let canonical = cargo_install_strategy(PackageChannel::Canonical);
        let compatibility = cargo_install_strategy(PackageChannel::Compatibility);
        assert_eq!(canonical.executable_name, CANONICAL_EXECUTABLE);
        assert_eq!(compatibility.executable_name, CANONICAL_EXECUTABLE);
    }

    #[test]
    fn pip_channels_resolve_to_same_canonical_executable() {
        let canonical = pip_install_strategy(PackageChannel::Canonical);
        let compatibility = pip_install_strategy(PackageChannel::Compatibility);
        assert_eq!(canonical.executable_name, CANONICAL_EXECUTABLE);
        assert_eq!(compatibility.executable_name, CANONICAL_EXECUTABLE);
    }

    #[test]
    fn no_secondary_executable_conflicts_for_supported_strategies() {
        let strategies = vec![
            cargo_install_strategy(PackageChannel::Canonical),
            cargo_install_strategy(PackageChannel::Compatibility),
            pip_install_strategy(PackageChannel::Canonical),
            pip_install_strategy(PackageChannel::Compatibility),
        ];
        assert!(!has_secondary_executable_conflict(&strategies));
    }

    #[test]
    fn path_binary_discovery_respects_path_order() {
        let temp = TempDir::new().expect("tempdir");
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        std::fs::create_dir_all(&first).expect("first");
        std::fs::create_dir_all(&second).expect("second");
        std::fs::write(first.join(CANONICAL_EXECUTABLE), b"#!/bin/sh\n").expect("write first");
        std::fs::write(second.join(CANONICAL_EXECUTABLE), b"#!/bin/sh\n")
            .expect("write second");
        let path_value = std::env::join_paths([&first, &second]).expect("join");
        let discovered = discover_path_binaries(path_value.to_str().expect("utf-8 path"));
        assert_eq!(discovered.len(), 2);
        assert!(discovered[0].contains("first"));
    }

    #[test]
    fn install_health_report_flags_shadowing_and_duplicate_installs() {
        let temp = TempDir::new().expect("tempdir");
        let pip_like = temp.path().join("python-site-packages-bin");
        let cargo_like = temp.path().join(".cargo-bin");
        std::fs::create_dir_all(&pip_like).expect("pip dir");
        std::fs::create_dir_all(&cargo_like).expect("cargo dir");
        std::fs::write(pip_like.join(CANONICAL_EXECUTABLE), b"#!/bin/sh\n").expect("write pip");
        std::fs::write(cargo_like.join(CANONICAL_EXECUTABLE), b"#!/bin/sh\n")
            .expect("write cargo");
        let path_value = std::env::join_paths([&pip_like, &cargo_like]).expect("join");

        let report = install_health_report(
            path_value.to_str().expect("utf-8 path"),
            None,
            Some("1.0.0"),
            "1.0.1",
        );
        assert!(report.has_duplicate_installs);
        assert!(report.has_mismatched_wheel_binary_versions);
    }

    #[test]
    fn active_binary_prefers_explicit_override() {
        let report = install_health_report("", Some("/custom/bin/bijux"), None, "1.0.0");
        assert_eq!(report.active_binary, Some("/custom/bin/bijux".to_string()));
    }

    #[test]
    fn completion_scripts_are_available_for_supported_shells() {
        assert!(completion_script(CompletionShell::Bash).contains("complete"));
        assert!(completion_script(CompletionShell::Zsh).contains("#compdef"));
        assert!(completion_script(CompletionShell::Fish).contains("complete -c"));
        assert!(completion_script(CompletionShell::PowerShell).contains("Register-ArgumentCompleter"));
    }

    #[test]
    fn post_install_hint_mentions_verification_commands() {
        let hint = post_install_hint("/usr/local/bin/bijux");
        assert!(hint.contains("bijux version"));
        assert!(hint.contains("bijux cli doctor"));
    }

    #[test]
    fn first_run_setup_is_idempotent() {
        let temp = TempDir::new().expect("tempdir");
        let first = initialize_first_run_state(temp.path()).expect("first run");
        let second = initialize_first_run_state(temp.path()).expect("second run");
        assert!(first);
        assert!(!second);
    }

    #[test]
    fn detects_legacy_installer_conflicts() {
        let temp = TempDir::new().expect("tempdir");
        let dir = temp.path().join("bin");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join("bijux.py"), b"#!/usr/bin/env python\n").expect("write legacy");
        let path_value = std::env::join_paths([&dir]).expect("join");
        let conflicts = legacy_installer_conflicts(path_value.to_str().expect("utf-8 path"));
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("bijux.py"));
    }

    #[test]
    fn no_network_environment_preserves_path_resolution() {
        let report = install_health_report("", None, None, "1.0.0");
        assert!(report.path_binaries.is_empty());
    }

    #[test]
    fn offline_plugin_registry_operation_is_stable_for_install_diagnostics() {
        let report = install_health_report("", Some("/tmp/bijux"), None, "1.0.0");
        assert_eq!(report.active_binary.as_deref(), Some("/tmp/bijux"));
    }

    #[test]
    fn read_only_environment_does_not_break_idempotent_check_when_marker_exists() {
        let temp = TempDir::new().expect("tempdir");
        let marker = temp.path().join(".first-run-ready");
        std::fs::write(&marker, b"ready").expect("write marker");
        let result = initialize_first_run_state(temp.path()).expect("idempotent check");
        assert!(!result);
    }

    #[test]
    fn nonstandard_home_paths_are_supported() {
        let temp = TempDir::new().expect("tempdir");
        let home = temp.path().join("XDG DATA HOME");
        std::fs::create_dir_all(&home).expect("mkdir");
        let initialized = initialize_first_run_state(&home).expect("initialize");
        assert!(initialized);
    }

    #[test]
    fn symlinked_binary_paths_are_detected() {
        let temp = TempDir::new().expect("tempdir");
        let target_dir = temp.path().join("target-bin");
        let link_dir = temp.path().join("link-bin");
        std::fs::create_dir_all(&target_dir).expect("mkdir target");
        std::fs::create_dir_all(&link_dir).expect("mkdir link");
        let binary = target_dir.join(CANONICAL_EXECUTABLE);
        std::fs::write(&binary, b"#!/bin/sh\n").expect("write binary");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&binary, link_dir.join(CANONICAL_EXECUTABLE)).expect("symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&binary, link_dir.join(CANONICAL_EXECUTABLE)).expect("symlink");
        let path_value = std::env::join_paths([&link_dir]).expect("join");
        let discovered = discover_path_binaries(path_value.to_str().expect("utf-8 path"));
        assert_eq!(discovered.len(), 1);
    }

    #[test]
    fn windows_path_override_is_respected() {
        let report = install_health_report(
            "",
            Some(r"C:\Program Files\Bijux\bijux.exe"),
            None,
            "1.0.0",
        );
        assert_eq!(
            report.active_binary.as_deref(),
            Some(r"C:\Program Files\Bijux\bijux.exe")
        );
    }

    #[test]
    #[cfg(unix)]
    fn read_only_state_directory_reports_error_on_first_write() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new().expect("tempdir");
        let dir = temp.path().join("readonly");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555))
            .expect("set readonly");
        let result = initialize_first_run_state(&dir);
        assert!(result.is_err());
    }
}
