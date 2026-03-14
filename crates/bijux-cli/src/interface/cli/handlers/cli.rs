//! `cli` command handlers.

use std::env;
use std::path::Path;
use serde_json::{json, Value};

use crate::api::config::validate_config_file;
use crate::api::version::{runtime_semver, runtime_version_info};
use crate::features::diagnostics::state_paths::{state_diagnostics, ResolvedStatePaths};
use crate::features::install::{
    completion_script, detect_shell, install_health_report, post_install_hint, CompletionShell,
};
use crate::features::plugins::{
    compatibility_warnings, list_plugins, plugin_doctor, plugin_origin_metadata,
};
use crate::routing::registry::RouteRegistry;

fn completion_shell_name(shell: CompletionShell) -> &'static str {
    match shell {
        CompletionShell::Bash => "bash",
        CompletionShell::Zsh => "zsh",
        CompletionShell::Fish => "fish",
        CompletionShell::PowerShell => "powershell",
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

pub(crate) fn completion_report() -> Value {
    let active_shell =
        detect_shell(env::var("SHELL").ok().as_deref()).unwrap_or(CompletionShell::Bash);
    let supported_shells = [
        CompletionShell::Bash,
        CompletionShell::Zsh,
        CompletionShell::Fish,
        CompletionShell::PowerShell,
    ]
    .into_iter()
    .map(completion_shell_name)
    .collect::<Vec<_>>();

    json!({
        "status": "ok",
        "active_shell": completion_shell_name(active_shell),
        "supported_shells": supported_shells,
        "script": completion_script(active_shell),
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

    if install.get("has_path_shadowing").and_then(Value::as_bool) == Some(true) {
        issues.push(json!({
            "area": "install",
            "severity": "warning",
            "message": "multiple bijux binaries are visible on PATH",
        }));
    }

    let status = if issues.iter().any(|item| item.get("severity") == Some(&json!("error"))) {
        "degraded"
    } else {
        "ok"
    };

    json!({
        "status": status,
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
    let install_status = if install.get("has_path_shadowing").and_then(Value::as_bool) == Some(true)
        || install.get("has_duplicate_installs").and_then(Value::as_bool) == Some(true)
    {
        "warning"
    } else {
        "ok"
    };
    checks.push(json!({
        "name": "install",
        "status": install_status,
        "message": "runtime install paths evaluated",
        "details": install,
    }));

    let state = state_diagnostics(paths);
    let state_issue_count = state
        .get("issues")
        .and_then(Value::as_array)
        .map_or(0, std::vec::Vec::len);
    checks.push(json!({
        "name": "state",
        "status": if state_issue_count == 0 { "ok" } else { "warning" },
        "message": "state files and rollback artifacts evaluated",
        "issue_count": state_issue_count,
    }));

    let issues = checks
        .iter()
        .filter(|check| check["status"] != "ok")
        .cloned()
        .collect::<Vec<_>>();

    json!({
        "status": if issues.iter().any(|item| item["status"] == "error") {
            "degraded"
        } else if issues.is_empty() {
            "ok"
        } else {
            "warning"
        },
        "checks": checks,
        "issues": issues,
    })
}

pub(crate) fn docs_inventory_report() -> Value {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let docs_root = workspace_root.join("docs");
    let references = [
        ("overview", "README.md"),
        ("first-run", "docs/01-introduction/first-run.md"),
        ("plugins", "docs/03-user-guide/plugins-and-extensions.md"),
        ("command-surface", "docs/06-reference/command-surface.md"),
        ("plugin-contracts", "docs/07-contracts/plugin-contracts.md"),
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

    json!({
        "status": if docs_root.exists() { "ok" } else { "warning" },
        "site_url": "https://bijux.github.io/bijux-cli/",
        "local_docs_root": docs_root,
        "local_docs_available": docs_root.exists(),
        "references": references,
    })
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
                .map(|_| "config file parsed successfully".to_string())
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
        [a, b] if a == "cli" && b == "doctor" => {
            let install = install_report_payload();
            Some(json!({
                "status": "healthy",
                "checks": ["routing", "output", "config", "install"],
                "install": {
                    "has_path_shadowing": install["has_path_shadowing"],
                    "has_duplicate_installs": install["has_duplicate_installs"],
                    "stale_wrapper_scripts": install["stale_wrapper_scripts"],
                    "legacy_installer_conflicts": install["legacy_installer_conflicts"].as_array().is_some_and(|items| !items.is_empty()),
                    "legacy_installer_conflict_paths": install["legacy_installer_conflicts"],
                    "has_mismatched_wheel_binary_versions": install["has_mismatched_wheel_binary_versions"],
                }
            }))
        }
        [a, b] if a == "cli" && b == "repl" => {
            Some(json!({"status": "ready", "mode": "repl", "history_file": paths.history_file}))
        }
        [a, b] if a == "cli" && b == "completion" => Some(completion_report()),
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
            let route_sources: Vec<Value> = registry
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
            Some(json!({
                "status": "ok",
                "reserved_namespaces": registry.route_tree(),
                "builtins": registry.built_in_paths(),
                "route_sources": route_sources,
                "alias_rewrites": registry.alias_rewrites().into_iter().map(|(alias, canonical)| {
                    let alias_segments: Vec<String> = alias.segments.into_iter().map(|s| s.0).collect();
                    let canonical_segments: Vec<String> = canonical.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "alias": alias_segments,
                        "canonical": canonical_segments,
                        "source": "compatibility-alias",
                    })
                }).collect::<Vec<_>>(),
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
        [a, b] if a == "cli" && b == "status" => Some(runtime_status_report(paths, plugin_registry_path)),
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
