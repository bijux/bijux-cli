#![forbid(unsafe_code)]
//! Installation and distribution surfaces.

mod compatibility;
mod completion;
mod diagnostics;
mod io;
mod metadata;
mod paths;
pub mod query;
mod state;

#[allow(unused_imports)]
pub use compatibility::{
    default_compatibility_paths, discover_compatibility_paths, load_compatibility_config,
    parse_compatibility_config, write_compatibility_config, CompatibilityConfig,
    CompatibilityError, CompatibilityPaths, PathOverrides, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
    ENV_PLUGINS_PATH,
};
#[allow(unused_imports)]
pub(crate) use completion::{completion_file_path, completion_script, detect_shell};
#[allow(unused_imports)]
pub use completion::{post_install_hint, CompletionShell};
#[allow(unused_imports)]
pub use diagnostics::{install_health_report, InstallHealthReport};
#[allow(unused_imports)]
pub use io::atomic_write_text;
#[allow(unused_imports)]
pub use metadata::{
    canonical_crate_name, cargo_install_strategy, install_target_aliases, pip_install_strategy,
    resolve_install_target, Ecosystem, InstallStrategy, InstallTarget, CANONICAL_EXECUTABLE,
};
#[allow(unused_imports)]
pub use paths::{
    detect_stale_wrapper_scripts, discover_named_path_binaries, discover_path_binaries,
    initialize_first_run_state, legacy_installer_conflicts, resolve_active_binary,
};
#[allow(unused_imports)]
pub use state::{
    acquire_state_lock, ensure_history_file, ensure_plugins_dir, run_config_migrations,
    StateLockGuard,
};

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::completion::{completion_file_path, completion_script, detect_shell};
    use super::*;
    use tempfile::TempDir;

    fn test_executable_name() -> String {
        let extension = std::env::consts::EXE_EXTENSION;
        if extension.is_empty() {
            CANONICAL_EXECUTABLE.to_string()
        } else {
            format!("{CANONICAL_EXECUTABLE}.{extension}")
        }
    }

    fn write_executable(path: &Path, content: &[u8]) {
        std::fs::write(path, content).expect("write executable");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(path, perms).expect("chmod +x");
        }
    }

    #[test]
    fn cargo_install_strategy_uses_canonical_public_package() {
        let canonical = cargo_install_strategy();
        assert_eq!(canonical.ecosystem, Ecosystem::Cargo);
        assert_eq!(canonical.package_name, "bijux-cli");
        assert_eq!(canonical.executable_name, CANONICAL_EXECUTABLE);
    }

    #[test]
    fn pip_install_strategy_uses_canonical_public_package() {
        let canonical = pip_install_strategy();
        assert_eq!(canonical.ecosystem, Ecosystem::Pip);
        assert_eq!(canonical.package_name, "bijux-cli");
        assert_eq!(canonical.executable_name, CANONICAL_EXECUTABLE);
    }

    #[test]
    fn path_binary_discovery_respects_path_order() {
        let temp = TempDir::new().expect("tempdir");
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        std::fs::create_dir_all(&first).expect("first");
        std::fs::create_dir_all(&second).expect("second");
        let executable = test_executable_name();
        write_executable(&first.join(&executable), b"#!/bin/sh\n");
        write_executable(&second.join(&executable), b"#!/bin/sh\n");
        let path_value = std::env::join_paths([&first, &second]).expect("join");
        let discovered = discover_path_binaries(path_value.to_str().expect("utf-8 path"));
        assert_eq!(discovered.len(), 2);
        assert!(discovered[0].contains("first"));
        assert!(discovered[1].contains("second"));
    }

    #[test]
    fn named_path_binary_discovery_supports_app_alias_binary_names() {
        let temp = TempDir::new().expect("tempdir");
        let shim_bin = temp.path().join("shim-bin");
        std::fs::create_dir_all(&shim_bin).expect("shim bin");
        write_executable(&shim_bin.join("bijux-workflow"), b"#!/bin/sh\n");
        let path_value = std::env::join_paths([&shim_bin]).expect("join path");

        let discovered = discover_named_path_binaries(
            path_value.to_str().expect("utf-8 path"),
            "bijux-workflow",
        );
        assert_eq!(discovered.len(), 1);
        assert!(discovered[0].ends_with("bijux-workflow"));
    }

    #[cfg(unix)]
    #[test]
    fn path_binary_discovery_ignores_non_executable_files() {
        let temp = TempDir::new().expect("tempdir");
        let bin = temp.path().join("bin");
        std::fs::create_dir_all(&bin).expect("mkdir bin");
        let executable = test_executable_name();
        std::fs::write(bin.join(&executable), b"#!/bin/sh\n").expect("write file");
        let path_value = std::env::join_paths([&bin]).expect("join path");

        let discovered = discover_path_binaries(path_value.to_str().expect("utf-8 path"));
        assert!(discovered.is_empty(), "non-executable file should not be discovered");
    }

    #[cfg(windows)]
    #[test]
    fn path_binary_discovery_finds_windows_executable_suffix() {
        let temp = TempDir::new().expect("tempdir");
        let bin = temp.path().join("bin");
        std::fs::create_dir_all(&bin).expect("mkdir bin");
        std::fs::write(bin.join("bijux.exe"), b"stub").expect("write exe");
        let path_value = std::env::join_paths([&bin]).expect("join path");

        let discovered = discover_path_binaries(path_value.to_str().expect("utf-8 path"));
        assert_eq!(discovered.len(), 1);
        assert!(discovered[0].ends_with("bijux.exe"));
    }

    #[test]
    fn install_health_report_flags_shadowing_and_duplicate_installs() {
        let temp = TempDir::new().expect("tempdir");
        let pip_like = temp.path().join("python-site-packages-bin");
        let cargo_like = temp.path().join(".cargo-bin");
        std::fs::create_dir_all(&pip_like).expect("pip dir");
        std::fs::create_dir_all(&cargo_like).expect("cargo dir");
        let executable = test_executable_name();
        write_executable(&pip_like.join(&executable), b"#!/bin/sh\n");
        write_executable(&cargo_like.join(&executable), b"#!/bin/sh\n");
        let path_value = std::env::join_paths([&pip_like, &cargo_like]).expect("join");

        let report = install_health_report(
            path_value.to_str().expect("utf-8 path"),
            None,
            Some("1.0.0"),
            "1.0.1",
        );
        assert!(report.has_path_shadowing);
        assert!(report.has_duplicate_installs);
        assert!(report.has_mismatched_wheel_binary_versions);
        assert_eq!(report.path_binaries.len(), 2);
    }

    #[test]
    fn mixed_pip_and_cargo_install_ambiguity_is_detected() {
        let temp = TempDir::new().expect("tempdir");
        let pip_bin = temp.path().join("venv-site-packages-bin");
        let cargo_bin = temp.path().join(".cargo/bin");
        std::fs::create_dir_all(&pip_bin).expect("pip dir");
        std::fs::create_dir_all(&cargo_bin).expect("cargo dir");
        let executable = test_executable_name();
        write_executable(&pip_bin.join(&executable), b"#!/bin/sh\n");
        write_executable(&cargo_bin.join(&executable), b"#!/bin/sh\n");
        let path_value = std::env::join_paths([&pip_bin, &cargo_bin]).expect("join");

        let report =
            install_health_report(path_value.to_str().expect("utf-8 path"), None, None, "1.0.0");

        assert!(report.has_path_shadowing);
        assert!(report.has_duplicate_installs);
        assert!(report
            .active_binary
            .as_deref()
            .is_some_and(|value| value.contains("venv-site-packages-bin")));
    }

    #[test]
    fn stale_wrapper_scripts_are_detected_in_install_health() {
        let temp = TempDir::new().expect("tempdir");
        let wrappers = temp.path().join("wrappers");
        std::fs::create_dir_all(&wrappers).expect("wrapper dir");
        write_executable(&wrappers.join("bijux.sh"), b"#!/bin/sh\nexec /missing/bijux\n");
        let path_value = std::env::join_paths([&wrappers]).expect("join path");

        let report =
            install_health_report(path_value.to_str().expect("utf-8 path"), None, None, "1.0.0");

        assert!(!report.stale_wrapper_scripts.is_empty());
    }

    #[test]
    fn stale_wrapper_cmd_scripts_are_detected_in_install_health() {
        let temp = TempDir::new().expect("tempdir");
        let wrappers = temp.path().join("wrappers");
        std::fs::create_dir_all(&wrappers).expect("wrapper dir");
        write_executable(&wrappers.join("bijux.cmd"), b"@echo off\r\n");
        let path_value = std::env::join_paths([&wrappers]).expect("join path");

        let report =
            install_health_report(path_value.to_str().expect("utf-8 path"), None, None, "1.0.0");

        assert!(report.stale_wrapper_scripts.iter().any(|entry| entry.ends_with("bijux.cmd")));
    }

    #[test]
    fn wheel_runtime_version_mismatch_is_reported() {
        let report = install_health_report("", None, Some("1.0.0"), "2.0.0");
        assert!(report.has_mismatched_wheel_binary_versions);
    }

    #[test]
    fn active_binary_ignores_blank_override_and_falls_back_to_discovered_path() {
        let temp = TempDir::new().expect("tempdir");
        let dir = temp.path().join("bin");
        std::fs::create_dir_all(&dir).expect("mkdir");
        let executable = test_executable_name();
        write_executable(&dir.join(&executable), b"#!/bin/sh\n");
        let path_value = std::env::join_paths([&dir]).expect("join path");
        let report = install_health_report(
            path_value.to_str().expect("utf-8 path"),
            Some("   "),
            None,
            "1.0.0",
        );
        assert!(report.active_binary.as_deref().is_some_and(|value| value.ends_with(&executable)));
    }

    #[test]
    fn completion_scripts_are_available_for_supported_shells() {
        assert!(completion_script(CompletionShell::Bash).contains("complete"));
        assert!(completion_script(CompletionShell::Zsh).contains("#compdef"));
        assert!(completion_script(CompletionShell::Fish).contains("complete -c"));
        assert!(
            completion_script(CompletionShell::PowerShell).contains("Register-ArgumentCompleter")
        );
    }

    #[test]
    fn post_install_hint_handles_missing_binary_path_without_dropping_guidance() {
        let hint = post_install_hint("");
        assert!(hint.contains("bijux version"));
        assert!(hint.contains("bijux doctor"));
        assert!(hint.contains("Installed `bijux` at "));
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
    fn missing_install_state_directory_is_created_on_first_run() {
        let temp = TempDir::new().expect("tempdir");
        let missing = temp.path().join("missing-state-root");
        assert!(!missing.exists());

        let created = initialize_first_run_state(&missing).expect("create missing state root");

        assert!(created);
        assert!(missing.exists());
        assert!(missing.join(".first-run-ready").exists());
    }

    #[test]
    fn detects_legacy_installer_conflicts() {
        let temp = TempDir::new().expect("tempdir");
        let dir = temp.path().join("bin");
        std::fs::create_dir_all(&dir).expect("mkdir");
        write_executable(&dir.join("bijux.py"), b"#!/usr/bin/env python\n");
        let path_value = std::env::join_paths([&dir]).expect("join");
        let conflicts = legacy_installer_conflicts(path_value.to_str().expect("utf-8 path"));
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("bijux.py"));
    }

    #[cfg(unix)]
    #[test]
    fn non_executable_legacy_artifacts_are_not_counted_as_conflicts() {
        let temp = TempDir::new().expect("tempdir");
        let dir = temp.path().join("bin");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join("bijux.py"), b"#!/usr/bin/env python\n").expect("write legacy");
        let path_value = std::env::join_paths([&dir]).expect("join");
        let conflicts = legacy_installer_conflicts(path_value.to_str().expect("utf-8 path"));
        assert!(conflicts.is_empty());
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
    fn symlinked_binary_paths_are_detected() {
        let temp = TempDir::new().expect("tempdir");
        let target_dir = temp.path().join("target-bin");
        let link_dir = temp.path().join("link-bin");
        std::fs::create_dir_all(&target_dir).expect("mkdir target");
        std::fs::create_dir_all(&link_dir).expect("mkdir link");
        let executable = test_executable_name();
        let binary = target_dir.join(&executable);
        write_executable(&binary, b"#!/bin/sh\n");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&binary, link_dir.join(&executable)).expect("symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&binary, link_dir.join(&executable)).expect("symlink");
        let path_value = std::env::join_paths([&link_dir]).expect("join");
        let discovered = discover_path_binaries(path_value.to_str().expect("utf-8 path"));
        assert_eq!(discovered.len(), 1);
    }

    #[test]
    fn windows_path_override_preserves_whitespace_without_truncation() {
        let report =
            install_health_report("", Some(r"  C:\Program Files\Bijux\bijux.exe  "), None, "1.0.0");
        assert_eq!(report.active_binary.as_deref(), Some(r"  C:\Program Files\Bijux\bijux.exe  "));
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
        let probe = dir.join(".permission-probe");
        if std::fs::write(&probe, b"probe").is_ok() {
            let _ = std::fs::remove_file(&probe);
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755))
                .expect("restore writable");
            return;
        }
        let result = initialize_first_run_state(&dir);
        assert!(result.is_err());
    }

    #[test]
    fn default_paths_match_python_expectations() {
        let home = std::path::PathBuf::from("/tmp/home");
        let paths = default_compatibility_paths(&home);
        assert_eq!(paths.config_file, home.join(".bijux/.env"));
        assert_eq!(paths.history_file, home.join(".bijux/.history"));
        assert_eq!(paths.plugins_dir, home.join(".bijux/.plugins"));
    }

    #[test]
    fn linux_path_resolution_is_supported() {
        let home = std::path::PathBuf::from("/home/bijan");
        let resolved = discover_compatibility_paths(
            Some(&home),
            &PathOverrides::default(),
            &std::collections::HashMap::new(),
            &CompatibilityConfig::default(),
        )
        .expect("resolve");
        assert_eq!(resolved.config_file, home.join(".bijux/.env"));
    }

    #[test]
    fn macos_path_resolution_is_supported() {
        let home = std::path::PathBuf::from("/Users/bijan");
        let resolved = discover_compatibility_paths(
            Some(&home),
            &PathOverrides::default(),
            &std::collections::HashMap::new(),
            &CompatibilityConfig::default(),
        )
        .expect("resolve");
        assert_eq!(resolved.history_file, home.join(".bijux/.history"));
    }

    #[test]
    fn windows_path_resolution_is_supported() {
        let home = std::path::PathBuf::from(r"C:\Users\bijan");
        let mut env_map = std::collections::HashMap::new();
        env_map.insert(ENV_PLUGINS_PATH.to_string(), r"C:\Users\bijan\.bijux\.plugins".to_string());
        let resolved = discover_compatibility_paths(
            Some(&home),
            &PathOverrides::default(),
            &env_map,
            &CompatibilityConfig::default(),
        )
        .expect("resolve");
        assert!(resolved.plugins_dir.to_string_lossy().contains(r"C:\Users\bijan\.bijux\.plugins"));
    }

    #[test]
    fn home_override_behavior_is_supported() {
        let home = std::path::PathBuf::from("/override/home");
        let mut env_map = std::collections::HashMap::new();
        env_map.insert(ENV_CONFIG_PATH.to_string(), "cfg/custom.env".to_string());
        let resolved = discover_compatibility_paths(
            Some(&home),
            &PathOverrides::default(),
            &env_map,
            &CompatibilityConfig::default(),
        )
        .expect("resolve");
        assert_eq!(resolved.config_file, home.join("cfg/custom.env"));
    }

    #[test]
    fn xdg_style_home_paths_are_supported() {
        let home = std::path::PathBuf::from("/home/bijan/.local/share");
        let resolved = discover_compatibility_paths(
            Some(&home),
            &PathOverrides::default(),
            &std::collections::HashMap::new(),
            &CompatibilityConfig::default(),
        )
        .expect("resolve");
        assert_eq!(resolved.config_file, home.join(".bijux/.env"));
    }

    #[test]
    fn completion_file_paths_are_generated() {
        let home = std::path::PathBuf::from("/tmp/home");
        let bash = completion_file_path(CompletionShell::Bash, &home);
        let zsh = completion_file_path(CompletionShell::Zsh, &home);
        let fish = completion_file_path(CompletionShell::Fish, &home);
        let powershell = completion_file_path(CompletionShell::PowerShell, &home);
        assert!(bash.to_string_lossy().contains(".bash_completion.d"));
        assert!(bash.to_string_lossy().ends_with("/bijux"));
        assert!(zsh.to_string_lossy().contains(".zsh"));
        assert!(zsh.to_string_lossy().ends_with("/_bijux"));
        assert!(fish.to_string_lossy().contains(".config/fish/completions"));
        assert!(fish.to_string_lossy().ends_with("/bijux.fish"));
        assert!(powershell.to_string_lossy().contains(".config/powershell"));
        assert!(powershell.to_string_lossy().contains("Microsoft.PowerShell_profile.ps1"));
    }

    #[test]
    fn shell_detection_rejects_unknown_shell_values() {
        assert_eq!(detect_shell(Some("/bin/bash")), Some(CompletionShell::Bash));
        assert_eq!(detect_shell(Some("/bin/zsh")), Some(CompletionShell::Zsh));
        assert_eq!(detect_shell(Some("/usr/bin/fish")), Some(CompletionShell::Fish));
        assert_eq!(detect_shell(Some("/usr/local/bin/pwsh")), Some(CompletionShell::PowerShell));
        assert_eq!(detect_shell(Some("powershell.exe")), None);
        assert_eq!(detect_shell(Some("/usr/bin/unknown")), None);
    }

    #[test]
    fn completion_target_write_fails_when_parent_is_not_directory() {
        let temp = TempDir::new().expect("tempdir");
        let blocker = temp.path().join("not-a-directory");
        std::fs::write(&blocker, b"blocker").expect("write blocker");
        let target = blocker.join("completions").join("bijux");
        let parent = target.parent().expect("parent");

        let mkdir = std::fs::create_dir_all(parent);
        assert!(mkdir.is_err());
        let write = std::fs::write(&target, completion_script(CompletionShell::Bash));
        assert!(write.is_err());
    }

    #[test]
    fn install_strategy_names_match_between_pip_and_cargo() {
        let cargo_canonical = cargo_install_strategy();
        let pip_canonical = pip_install_strategy();
        assert_eq!(cargo_canonical.package_name, pip_canonical.package_name);
        assert_eq!(cargo_canonical.executable_name, pip_canonical.executable_name);
    }

    #[test]
    fn install_target_aliases_resolve_to_canonical_packages() {
        let cli = resolve_install_target("cli").expect("cli target");
        assert_eq!(cli.target_name, "cli");
        assert_eq!(cli.strategy.package_name, "bijux-cli");
        assert_eq!(cli.strategy.executable_name, "bijux");

        let dev_cli = resolve_install_target("dev-cli").expect("dev-cli target");
        assert_eq!(dev_cli.target_name, "dev-cli");
        assert_eq!(dev_cli.strategy.package_name, "bijux-dev-cli");
        assert_eq!(dev_cli.strategy.executable_name, "bijux-dev-cli");

        let atlas = resolve_install_target("atlas").expect("atlas target");
        assert_eq!(atlas.target_name, "atlas");
        assert_eq!(atlas.strategy.package_name, "bijux-atlas");
        assert_eq!(atlas.strategy.executable_name, "bijux-atlas");

        let dev_atlas = resolve_install_target("bijux-dev-atlas").expect("dev-atlas target");
        assert_eq!(dev_atlas.target_name, "dev-atlas");
        assert_eq!(dev_atlas.strategy.package_name, "bijux-dev-atlas");
        assert_eq!(dev_atlas.strategy.executable_name, "bijux-dev-atlas");
    }

    #[test]
    fn install_target_alias_inventory_stays_short_and_canonical() {
        let aliases = install_target_aliases();
        assert!(aliases.contains(&"cli".to_string()));
        assert!(aliases.contains(&"dev-cli".to_string()));
        assert!(aliases.contains(&"atlas".to_string()));
        assert!(aliases.contains(&"dev-atlas".to_string()));
        assert!(!aliases.iter().any(|alias| alias.starts_with("bijux-")));
    }

    #[test]
    fn install_health_report_performance_is_within_sanity_budget() {
        let started = std::time::Instant::now();
        let report = install_health_report("", None, None, "1.0.0");
        assert!(started.elapsed() < std::time::Duration::from_millis(100));
        assert!(!report.has_path_shadowing);
        assert!(report.path_binaries.is_empty());
        assert!(report.active_binary.is_none());
    }
}
