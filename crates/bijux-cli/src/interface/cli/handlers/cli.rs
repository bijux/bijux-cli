//! `cli` command handlers.

use serde_json::{json, Value};
use std::env;
use std::path::Path;

use crate::api::config::validate_config_file;
use crate::api::version::{runtime_semver, runtime_version_info};
use crate::features::diagnostics::state_paths::{state_diagnostics, ResolvedStatePaths};
use crate::features::install::{
    completion_file_path, completion_script, detect_shell, install_health_report,
    post_install_hint, CompletionShell,
};
use crate::features::plugins::{
    compatibility_warnings, list_plugins, plugin_doctor, plugin_origin_metadata,
};
use crate::routing::registry::RouteRegistry;
use crate::shared::argv::command_option_value;

fn completion_shell_name(shell: CompletionShell) -> &'static str {
    match shell {
        CompletionShell::Bash => "bash",
        CompletionShell::Zsh => "zsh",
        CompletionShell::Fish => "fish",
        CompletionShell::PowerShell => "pwsh",
    }
}

fn install_report_payload() -> Value {
    let install_report = install_health_report(
        &env::var("PATH").unwrap_or_default(),
        env::var("BIJUX_BIN").ok().as_deref(),
        env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
        runtime_semver(),
    );
    json!({
        "active_binary": install_report.active_binary,
        "path_binaries": install_report.path_binaries,
        "has_path_shadowing": install_report.has_path_shadowing,
        "has_duplicate_installs": install_report.has_duplicate_installs,
        "stale_wrapper_scripts": install_report.stale_wrapper_scripts,
        "legacy_installer_conflicts": install_report.legacy_installer_conflicts,
        "has_mismatched_wheel_binary_versions": install_report.has_mismatched_wheel_binary_versions,
    })
}

fn install_warning_messages(install: &Value) -> Vec<&'static str> {
    [
        (
            install.get("has_path_shadowing").and_then(Value::as_bool) == Some(true),
            "multiple bijux binaries are visible on PATH",
        ),
        (
            install.get("has_duplicate_installs").and_then(Value::as_bool) == Some(true),
            "duplicate bijux installs were detected",
        ),
        (
            install.get("has_mismatched_wheel_binary_versions").and_then(Value::as_bool)
                == Some(true),
            "wheel and binary versions do not match",
        ),
        (
            install
                .get("stale_wrapper_scripts")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty()),
            "stale wrapper scripts were found",
        ),
        (
            install
                .get("legacy_installer_conflicts")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty()),
            "legacy installer conflicts were found",
        ),
    ]
    .into_iter()
    .filter_map(|(active, message)| active.then_some(message))
    .collect()
}

fn issue_status(issues: &[Value]) -> &'static str {
    if issues.iter().any(|item| {
        item.get("status") == Some(&json!("error")) || item.get("severity") == Some(&json!("error"))
    }) {
        "degraded"
    } else if issues.is_empty() {
        "ok"
    } else {
        "warning"
    }
}

fn resolve_completion_shell(argv: &[String]) -> (CompletionShell, &'static str) {
    if let Some(raw) = command_option_value(argv, &["cli", "completion"], "--shell") {
        if let Some(shell) = CompletionShell::from_cli_value(&raw) {
            return (shell, "explicit");
        }
    }

    if let Some(shell) = detect_shell(env::var("SHELL").ok().as_deref()) {
        return (shell, "detected");
    }

    (CompletionShell::Bash, "default")
}

pub(crate) fn runtime_error_payload(message: String) -> Value {
    json!({
        "status": "error",
        "message": message,
    })
}

pub(crate) fn completion_report(argv: &[String]) -> Value {
    let (active_shell, selection_source) = resolve_completion_shell(argv);
    let supported_shells = [
        CompletionShell::Bash,
        CompletionShell::Zsh,
        CompletionShell::Fish,
        CompletionShell::PowerShell,
    ]
    .into_iter()
    .map(completion_shell_name)
    .collect::<Vec<_>>();
    let target_file = env::var_os("HOME").map(|home| {
        completion_file_path(active_shell, Path::new(&home)).to_string_lossy().into_owned()
    });

    json!({
        "status": "ok",
        "active_shell": completion_shell_name(active_shell),
        "selection_source": selection_source,
        "supported_shells": supported_shells,
        "supported_platforms": ["linux", "macos"],
        "windows_supported": false,
        "target_file": target_file,
        "script": completion_script(active_shell),
    })
}

pub(crate) fn doctor_report(paths: &ResolvedStatePaths, plugin_registry_path: &Path) -> Value {
    let install = install_report_payload();
    let config_result = validate_config_file(&paths.config_file);
    let state = state_diagnostics(paths);
    let plugins = plugin_doctor(plugin_registry_path);

    let mut checks = Vec::<Value>::new();
    let mut issues = Vec::<Value>::new();

    let config_status = match &config_result {
        Ok(()) => "ok",
        Err(_) => "error",
    };
    let config_message = match config_result {
        Ok(()) if paths.config_file.exists() => "config file parsed successfully".to_string(),
        Ok(()) => "config file is absent and will be treated as empty".to_string(),
        Err(error) => error,
    };
    checks.push(json!({
        "name": "config",
        "status": config_status,
        "message": config_message,
    }));

    let install_warnings = install_warning_messages(&install);
    let install_status = if install_warnings.is_empty() { "ok" } else { "warning" };
    checks.push(json!({
        "name": "install",
        "status": install_status,
        "message": if install_warnings.is_empty() {
            "runtime install paths evaluated".to_string()
        } else {
            install_warnings.join("; ")
        },
    }));

    let state_issue_count =
        state.get("issues").and_then(Value::as_array).map_or(0, std::vec::Vec::len);
    checks.push(json!({
        "name": "state",
        "status": if state_issue_count == 0 { "ok" } else { "warning" },
        "message": "runtime state files evaluated",
        "issue_count": state_issue_count,
    }));

    match plugins {
        Ok(report) => {
            let status = if report.broken.is_empty() && report.incompatible.is_empty() {
                "ok"
            } else {
                "warning"
            };
            checks.push(json!({
                "name": "plugins",
                "status": status,
                "message": if status == "ok" {
                    "plugin registry health evaluated".to_string()
                } else {
                    "installed plugins need attention".to_string()
                },
                "installed": report.installed,
                "broken": report.broken,
                "incompatible": report.incompatible,
            }));
        }
        Err(error) => checks.push(json!({
            "name": "plugins",
            "status": "error",
            "message": error.to_string(),
        })),
    }

    issues.extend(checks.iter().filter(|check| check["status"] != "ok").cloned());
    if let Some(items) = state.get("issues").and_then(Value::as_array) {
        issues.extend(items.iter().cloned());
    }

    json!({
        "status": issue_status(&issues),
        "checks": checks,
        "install": {
            "has_path_shadowing": install["has_path_shadowing"],
            "has_duplicate_installs": install["has_duplicate_installs"],
            "stale_wrapper_scripts": install["stale_wrapper_scripts"],
            "legacy_installer_conflicts": install["legacy_installer_conflicts"].as_array().is_some_and(|items| !items.is_empty()),
            "legacy_installer_conflict_paths": install["legacy_installer_conflicts"],
            "has_mismatched_wheel_binary_versions": install["has_mismatched_wheel_binary_versions"],
        },
        "issues": issues,
    })
}

pub(crate) fn runtime_status_report(
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Value {
    let version = runtime_version_info();
    let plugins = plugin_doctor(plugin_registry_path);
    let state = state_diagnostics(paths);
    let install = install_report_payload();
    let mut issues = Vec::<Value>::new();

    if let Some(items) = state.get("issues").and_then(Value::as_array) {
        issues.extend(items.iter().cloned());
    }
    match &plugins {
        Ok(report) => {
            if !report.broken.is_empty() || !report.incompatible.is_empty() {
                issues.push(json!({
                    "area": "plugins",
                    "severity": "warning",
                    "broken": report.broken,
                    "incompatible": report.incompatible,
                    "message": "installed plugins need attention",
                }));
            }
        }
        Err(error) => issues.push(json!({
            "area": "plugins",
            "severity": "error",
            "message": error.to_string(),
        })),
    }

    for message in install_warning_messages(&install) {
        issues.push(json!({
            "area": "install",
            "severity": "warning",
            "message": message,
        }));
    }

    json!({
        "status": issue_status(&issues),
        "runtime": {
            "name": version.name,
            "version": version.version,
            "semver": version.semver,
            "source": version.source,
            "git_commit": version.git_commit,
            "git_dirty": version.git_dirty,
            "build_profile": version.build_profile,
        },
        "state": {
            "config": paths.config_file,
            "history": paths.history_file,
            "plugins": paths.plugins_dir,
            "plugin_registry": paths.plugin_registry_file,
            "path_resolution_warning": paths.compatibility_config_warning,
        },
        "plugins": match plugins {
            Ok(report) => json!({
                "installed": report.installed,
                "broken": report.broken,
                "incompatible": report.incompatible,
            }),
            Err(error) => json!({
                "status": "unavailable",
                "message": error.to_string(),
            }),
        },
        "install": install,
        "issues": issues,
    })
}

pub(crate) fn runtime_audit_report(
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Value {
    let mut checks = Vec::<Value>::new();

    let config_result = validate_config_file(&paths.config_file);
    let (config_status, config_message) = match config_result {
        Ok(()) => {
            let message = if paths.config_file.exists() {
                "config file parsed successfully"
            } else {
                "config file is absent and will be treated as empty"
            };
            ("ok", message.to_string())
        }
        Err(error) => ("error", error),
    };
    checks.push(json!({"name": "config", "status": config_status, "message": config_message}));

    match plugin_doctor(plugin_registry_path) {
        Ok(report) => {
            let status = if report.broken.is_empty() && report.incompatible.is_empty() {
                "ok"
            } else {
                "warning"
            };
            checks.push(json!({
                "name": "plugins",
                "status": status,
                "installed": report.installed,
                "broken": report.broken,
                "incompatible": report.incompatible,
                "message": "plugin registry health evaluated",
            }));
        }
        Err(error) => checks.push(json!({
            "name": "plugins",
            "status": "error",
            "message": error.to_string(),
        })),
    }

    let install = install_report_payload();
    let install_warnings = install_warning_messages(&install);
    let install_status = if install_warnings.is_empty() { "ok" } else { "warning" };
    checks.push(json!({
        "name": "install",
        "status": install_status,
        "message": if install_warnings.is_empty() {
            "runtime install paths evaluated".to_string()
        } else {
            install_warnings.join("; ")
        },
        "details": install,
    }));

    let state = state_diagnostics(paths);
    let state_issue_count =
        state.get("issues").and_then(Value::as_array).map_or(0, std::vec::Vec::len);
    checks.push(json!({
        "name": "state",
        "status": if state_issue_count == 0 { "ok" } else { "warning" },
        "message": "state files and rollback artifacts evaluated",
        "issue_count": state_issue_count,
    }));

    let issues = checks.iter().filter(|check| check["status"] != "ok").cloned().collect::<Vec<_>>();

    json!({
        "status": issue_status(&issues),
        "checks": checks,
        "issues": issues,
    })
}

fn docs_inventory_report_at(workspace_root: &Path) -> Value {
    let docs_root = workspace_root.join("docs");
    let references = [
        ("overview", "README.md"),
        ("cli-handbook", "docs/bijux-cli/index.md"),
        ("installation", "docs/bijux-cli/operations/installation-and-setup.md"),
        ("plugin-workflows", "docs/bijux-cli/interfaces/operator-workflows.md"),
        ("compatibility", "docs/bijux-cli/interfaces/compatibility-commitments.md"),
        ("quality-review", "docs/bijux-cli/quality/review-checklist.md"),
    ]
    .into_iter()
    .map(|(name, relative)| {
        let path = workspace_root.join(relative);
        json!({
            "name": name,
            "path": relative,
            "exists": path.exists(),
        })
    })
    .collect::<Vec<_>>();
    let missing_references = references
        .iter()
        .filter(|reference| reference.get("exists") != Some(&json!(true)))
        .map(|reference| {
            reference.get("path").and_then(Value::as_str).unwrap_or_default().to_string()
        })
        .collect::<Vec<_>>();
    let docs_available = docs_root.exists();

    json!({
        "status": if docs_available && missing_references.is_empty() { "ok" } else { "warning" },
        "site_url": "https://bijux.io/bijux-core/bijux-cli/",
        "local_docs_root": docs_root,
        "local_docs_available": docs_available,
        "references": references,
        "missing_references": missing_references,
    })
}

pub(crate) fn docs_inventory_report() -> Value {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    docs_inventory_report_at(&workspace_root)
}

pub(crate) fn self_test_report(
    paths: &ResolvedStatePaths,
    registry: &RouteRegistry,
    plugin_registry_path: &Path,
) -> Value {
    let route_check = !registry.built_in_paths().is_empty()
        && registry.route_tree().iter().any(|item| item.name.0 == "cli");
    let state_check = paths.plugin_registry_file.parent() == Some(paths.plugins_dir.as_path())
        && paths.memory_file.parent() == paths.config_file.parent();
    let config_check = validate_config_file(&paths.config_file);
    let plugin_check = list_plugins(plugin_registry_path);
    let completion_check = [
        CompletionShell::Bash,
        CompletionShell::Zsh,
        CompletionShell::Fish,
        CompletionShell::PowerShell,
    ]
    .into_iter()
    .all(|shell| !completion_script(shell).trim().is_empty());

    let checks = vec![
        json!({
            "name": "routing",
            "status": if route_check { "ok" } else { "error" },
            "message": "route registry builds and exposes core namespaces",
        }),
        json!({
            "name": "state-paths",
            "status": if state_check { "ok" } else { "error" },
            "message": "state path relationships are internally consistent",
        }),
        json!({
            "name": "config",
            "status": if config_check.is_ok() { "ok" } else { "error" },
            "message": config_check
                .as_ref()
                .map(|()| "config file parsed successfully".to_string())
                .unwrap_or_else(|error| error.clone()),
        }),
        json!({
            "name": "plugin-registry",
            "status": if plugin_check.is_ok() { "ok" } else { "error" },
            "message": plugin_check
                .as_ref()
                .map(|plugins| format!("loaded {} installed plugin records", plugins.len()))
                .unwrap_or_else(|error| error.to_string()),
        }),
        json!({
            "name": "completion",
            "status": if completion_check { "ok" } else { "error" },
            "message": "completion scripts render for supported shells",
        }),
    ];

    json!({
        "status": if checks.iter().all(|item| item["status"] == "ok") { "ok" } else { "degraded" },
        "checks": checks,
    })
}

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
    registry: &RouteRegistry,
    plugin_registry_path: &Path,
) -> Option<Value> {
    match normalized_path {
        [a, b] if a == "cli" && b == "version" => {
            let version = runtime_version_info();
            Some(json!({
                "name": version.name,
                "version": version.version,
                "semver": version.semver,
                "source": version.source,
                "git_commit": version.git_commit,
                "git_dirty": version.git_dirty,
                "build_profile": version.build_profile,
            }))
        }
        [a, b] if a == "cli" && b == "doctor" => Some(doctor_report(paths, plugin_registry_path)),
        [a, b] if a == "cli" && b == "repl" => Some(json!({
            "status": "ready",
            "mode": "interactive",
            "interactive": true,
            "history_file": paths.history_file,
            "message": "The process entrypoint launches the persistent REPL session loop.",
        })),
        [a, b] if a == "cli" && b == "completion" => Some(completion_report(argv)),
        [a, b] if a == "cli" && b == "inspect" => {
            let mut integrity_issues = Vec::<Value>::new();
            let plugin_origins = match plugin_origin_metadata(plugin_registry_path) {
                Ok(origins) => origins,
                Err(error) => {
                    integrity_issues.push(json!({
                        "source": "plugin-origin-metadata",
                        "error": error.to_string(),
                    }));
                    Vec::new()
                }
            };
            let compatibility = match compatibility_warnings(plugin_registry_path, runtime_semver())
            {
                Ok(warnings) => warnings,
                Err(error) => {
                    integrity_issues.push(json!({
                        "source": "compatibility-warnings",
                        "error": error.to_string(),
                    }));
                    Vec::new()
                }
            };
            let mut route_sources: Vec<Value> = registry
                .built_in_paths()
                .into_iter()
                .map(|path| {
                    let segments: Vec<String> = path.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "segments": segments,
                        "owner": "bijux-cli",
                        "source": "built-in",
                    })
                })
                .collect();
            route_sources.extend(
                registry.route_tree().into_iter().filter(|item| !item.reserved).map(|item| {
                    let source = if item.owner.starts_with("plugin-alias:") {
                        "plugin-alias"
                    } else {
                        "plugin"
                    };
                    json!({
                        "segments": [item.name.0],
                        "owner": item.owner,
                        "source": source,
                    })
                }),
            );
            Some(json!({
                "status": "ok",
                "reserved_namespaces": registry.route_tree(),
                "builtins": registry.built_in_paths(),
                "route_sources": route_sources,
                "alias_rewrites": registry
                    .alias_rewrites()
                    .into_iter()
                    .map(|(alias, canonical)| {
                        let alias_segments: Vec<String> =
                            alias.segments.into_iter().map(|s| s.0).collect();
                        let canonical_segments: Vec<String> =
                            canonical.segments.into_iter().map(|s| s.0).collect();
                        json!({
                            "alias": alias_segments,
                            "canonical": canonical_segments,
                            "source": "compatibility-alias",
                        })
                    })
                    .chain(registry.plugin_alias_rewrites().into_iter().map(|(alias, canonical)| {
                        let alias_segments: Vec<String> =
                            alias.segments.into_iter().map(|s| s.0).collect();
                        let canonical_segments: Vec<String> =
                            canonical.segments.into_iter().map(|s| s.0).collect();
                        json!({
                            "alias": alias_segments,
                            "canonical": canonical_segments,
                            "source": "plugin-alias",
                        })
                    }))
                    .collect::<Vec<_>>(),
                "plugin_origins": plugin_origins,
                "compatibility_warnings": compatibility,
                "integrity_status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
                "integrity_issues": integrity_issues,
                "contracts": {
                    "schemas": ["output-envelope-v1", "error-envelope-v1", "plugin-manifest-v2"],
                    "version": "v1",
                }
            }))
        }
        [a, b] if a == "cli" && b == "status" => {
            Some(runtime_status_report(paths, plugin_registry_path))
        }
        [a, b] if a == "cli" && b == "paths" => {
            let install = install_report_payload();
            let hint = install
                .get("active_binary")
                .and_then(Value::as_str)
                .map(post_install_hint)
                .unwrap_or_else(|| {
                    "Run `bijux version` and `bijux doctor` to verify your environment.".to_string()
                });
            Some(json!({
                "config": paths.config_file,
                "history": paths.history_file,
                "plugins": paths.plugins_dir,
                "path_resolution_warning": paths.compatibility_config_warning,
                "active_binary": install["active_binary"],
                "path_binaries": install["path_binaries"],
                "post_install_hint": hint
            }))
        }
        [a, b] if a == "cli" && b == "self-test" => {
            Some(self_test_report(paths, registry, plugin_registry_path))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{completion_report, doctor_report, runtime_audit_report, runtime_status_report};
    use crate::features::diagnostics::state_paths::ResolvedStatePaths;
    use crate::shared::telemetry::TEST_ENV_LOCK;

    #[test]
    fn completion_report_declares_supported_platform_contract() {
        let report = completion_report(&[
            "bijux".to_string(),
            "completion".to_string(),
            "--shell".to_string(),
            "pwsh".to_string(),
        ]);
        assert_eq!(report["supported_platforms"], serde_json::json!(["linux", "macos"]));
        assert_eq!(report["windows_supported"], serde_json::json!(false));
        assert_eq!(report["active_shell"], serde_json::json!("pwsh"));
        assert_eq!(report["selection_source"], serde_json::json!("explicit"));
        assert!(report["supported_shells"]
            .as_array()
            .expect("supported shells")
            .iter()
            .any(|shell| shell == "pwsh"));
    }

    #[test]
    fn doctor_report_degrades_when_install_surface_has_real_warnings() {
        let _guard = TEST_ENV_LOCK.lock().expect("env lock");
        let temp = tempdir().expect("temp dir");
        let bin_a = temp.path().join("bin-a");
        let bin_b = temp.path().join("bin-b");
        std::fs::create_dir_all(&bin_a).expect("bin-a");
        std::fs::create_dir_all(&bin_b).expect("bin-b");
        let path_a = bin_a.join("bijux");
        let path_b = bin_b.join("bijux");
        std::fs::write(&path_a, "#!/bin/sh\n").expect("bijux a");
        std::fs::write(&path_b, "#!/bin/sh\n").expect("bijux b");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&path_a, std::fs::Permissions::from_mode(0o755))
                .expect("chmod a");
            std::fs::set_permissions(&path_b, std::fs::Permissions::from_mode(0o755))
                .expect("chmod b");
        }

        let old_path = std::env::var_os("PATH");
        let old_bin = std::env::var_os("BIJUX_BIN");
        let joined_path = std::env::join_paths([&bin_a, &bin_b]).expect("join path");
        std::env::set_var("PATH", joined_path);
        std::env::set_var("BIJUX_BIN", &path_a);

        let paths = ResolvedStatePaths {
            config_file: temp.path().join("config.env"),
            history_file: temp.path().join("history.txt"),
            plugins_dir: temp.path().join("plugins"),
            plugin_registry_file: temp.path().join("plugins/registry.json"),
            memory_file: temp.path().join("memory.json"),
            compatibility_config_file: temp.path().join("compatibility.env"),
            compatibility_config_warning: None,
        };
        let report = doctor_report(&paths, &paths.plugin_registry_file);

        if let Some(value) = old_path {
            std::env::set_var("PATH", value);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(value) = old_bin {
            std::env::set_var("BIJUX_BIN", value);
        } else {
            std::env::remove_var("BIJUX_BIN");
        }

        assert_eq!(report["status"], serde_json::json!("warning"));
        assert_eq!(report["install"]["has_path_shadowing"], serde_json::json!(true));
    }

    #[test]
    fn runtime_status_report_warns_when_install_surface_has_real_warnings() {
        let _guard = TEST_ENV_LOCK.lock().expect("env lock");
        let temp = tempdir().expect("temp dir");
        let bin_a = temp.path().join("bin-a");
        let bin_b = temp.path().join("bin-b");
        std::fs::create_dir_all(&bin_a).expect("bin-a");
        std::fs::create_dir_all(&bin_b).expect("bin-b");
        let path_a = bin_a.join("bijux");
        let path_b = bin_b.join("bijux");
        std::fs::write(&path_a, "#!/bin/sh\n").expect("bijux a");
        std::fs::write(&path_b, "#!/bin/sh\n").expect("bijux b");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&path_a, std::fs::Permissions::from_mode(0o755))
                .expect("chmod a");
            std::fs::set_permissions(&path_b, std::fs::Permissions::from_mode(0o755))
                .expect("chmod b");
        }

        let old_path = std::env::var_os("PATH");
        let old_bin = std::env::var_os("BIJUX_BIN");
        let joined_path = std::env::join_paths([&bin_a, &bin_b]).expect("join path");
        std::env::set_var("PATH", joined_path);
        std::env::set_var("BIJUX_BIN", &path_a);

        let paths = ResolvedStatePaths {
            config_file: temp.path().join("config.env"),
            history_file: temp.path().join("history.txt"),
            plugins_dir: temp.path().join("plugins"),
            plugin_registry_file: temp.path().join("plugins/registry.json"),
            memory_file: temp.path().join("memory.json"),
            compatibility_config_file: temp.path().join("compatibility.env"),
            compatibility_config_warning: None,
        };
        let report = runtime_status_report(&paths, &paths.plugin_registry_file);

        if let Some(value) = old_path {
            std::env::set_var("PATH", value);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(value) = old_bin {
            std::env::set_var("BIJUX_BIN", value);
        } else {
            std::env::remove_var("BIJUX_BIN");
        }

        assert_eq!(report["status"], serde_json::json!("warning"));
        assert!(report["issues"].as_array().expect("issues").iter().any(|issue| issue["message"]
            == serde_json::json!("multiple bijux binaries are visible on PATH")));
    }

    #[test]
    fn runtime_audit_report_warns_for_all_install_surface_problems() {
        let _guard = TEST_ENV_LOCK.lock().expect("env lock");
        let temp = tempdir().expect("temp dir");
        let wrapper_bin = temp.path().join("wrapper-bin");
        let active_bin = temp.path().join("active-bin");
        std::fs::create_dir_all(&wrapper_bin).expect("wrapper bin");
        std::fs::create_dir_all(&active_bin).expect("active bin");
        let stale_wrapper = wrapper_bin.join("bijux.cmd");
        let legacy = active_bin.join("bijux.py");
        let active = active_bin.join("bijux");
        std::fs::write(&stale_wrapper, "#!/bin/sh\n").expect("stale wrapper");
        std::fs::write(&legacy, "#!/bin/sh\n").expect("legacy");
        std::fs::write(&active, "#!/bin/sh\n").expect("active");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&stale_wrapper, std::fs::Permissions::from_mode(0o755))
                .expect("chmod stale wrapper");
            std::fs::set_permissions(&legacy, std::fs::Permissions::from_mode(0o755))
                .expect("chmod legacy");
            std::fs::set_permissions(&active, std::fs::Permissions::from_mode(0o755))
                .expect("chmod active");
        }

        let old_path = std::env::var_os("PATH");
        let old_bin = std::env::var_os("BIJUX_BIN");
        let old_wheel = std::env::var_os("BIJUX_WHEEL_VERSION");
        let joined_path = std::env::join_paths([&wrapper_bin, &active_bin]).expect("join path");
        std::env::set_var("PATH", joined_path);
        std::env::set_var("BIJUX_BIN", &active);
        std::env::set_var("BIJUX_WHEEL_VERSION", "9.9.9");

        let paths = ResolvedStatePaths {
            config_file: temp.path().join("config.env"),
            history_file: temp.path().join("history.txt"),
            plugins_dir: temp.path().join("plugins"),
            plugin_registry_file: temp.path().join("plugins/registry.json"),
            memory_file: temp.path().join("memory.json"),
            compatibility_config_file: temp.path().join("compatibility.env"),
            compatibility_config_warning: None,
        };
        let report = runtime_audit_report(&paths, &paths.plugin_registry_file);

        if let Some(value) = old_path {
            std::env::set_var("PATH", value);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(value) = old_bin {
            std::env::set_var("BIJUX_BIN", value);
        } else {
            std::env::remove_var("BIJUX_BIN");
        }
        if let Some(value) = old_wheel {
            std::env::set_var("BIJUX_WHEEL_VERSION", value);
        } else {
            std::env::remove_var("BIJUX_WHEEL_VERSION");
        }

        assert_eq!(report["status"], serde_json::json!("warning"));
        let install_check = report["checks"]
            .as_array()
            .expect("checks")
            .iter()
            .find(|check| check["name"] == serde_json::json!("install"))
            .expect("install check");
        let message = install_check["message"].as_str().expect("install message");
        assert!(message.contains("stale wrapper scripts were found"));
        assert!(message.contains("legacy installer conflicts were found"));
        assert!(message.contains("wheel and binary versions do not match"));
    }
}
