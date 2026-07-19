//! `cli` command handlers.

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::api::config::validate_config_file;
use crate::api::version::{runtime_semver, runtime_version_info};
use crate::features::apps::{app_doctor_report, apps_doctor_report};
use crate::features::config::layered::schema_docs_report;
use crate::features::diagnostics::state_paths::{state_diagnostics, ResolvedStatePaths};
use crate::features::diagnostics::{registry_inventory, route_inventory, state_diagnostics_query};
use crate::features::install::{
    completion_file_path, completion_script, detect_shell, discover_named_path_binaries,
    install_health_report, post_install_hint, CompletionShell,
};
use crate::features::plugins::runtime::{detected_python_interpreters, resolve_python_interpreter};
use crate::features::plugins::{
    compatibility_warnings, list_plugins, plugin_doctor, plugin_origin_metadata,
};
use crate::routing::registry::RouteRegistry;
use crate::shared::argv::{command_has_flag, command_option_value, command_positionals};

const PYTHON_BRIDGE_DOCTOR_PROBE: &str = r#"import importlib, importlib.metadata, importlib.util, json
payload = {
    "module": "bijux_cli_py",
    "module_spec": None,
    "import_ok": False,
    "import_error": None,
    "package_version": None,
    "console_scripts": [],
}
spec = importlib.util.find_spec("bijux_cli_py")
if spec is not None:
    payload["module_spec"] = spec.origin
try:
    importlib.import_module("bijux_cli_py")
    payload["import_ok"] = True
except Exception as exc:
    payload["import_error"] = f"{type(exc).__name__}: {exc}"
for candidate in ("bijux-cli", "bijux_cli_py"):
    try:
        payload["package_version"] = importlib.metadata.version(candidate)
        break
    except importlib.metadata.PackageNotFoundError:
        pass
try:
    entry_points = importlib.metadata.entry_points()
    if hasattr(entry_points, "select"):
        selected = entry_points.select(group="console_scripts")
    else:
        selected = entry_points.get("console_scripts", [])
    payload["console_scripts"] = sorted(
        {
            ep.name: ep.value
            for ep in selected
            if ep.name == "bijux" or ep.value.startswith("bijux_cli_py")
        }.items()
    )
except Exception:
    payload["console_scripts"] = []
print(json.dumps(payload))
"#;

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

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "blocked" => 4,
        "error" => 3,
        "warning" => 2,
        "info" => 1,
        _ => 0,
    }
}

fn max_severity<'a>(severities: impl IntoIterator<Item = &'a str>) -> &'static str {
    let mut highest = 0;
    for severity in severities {
        highest = highest.max(severity_rank(severity));
    }
    match highest {
        4 => "blocked",
        3 => "error",
        2 => "warning",
        1 => "info",
        _ => "ok",
    }
}

fn status_from_severity(severity: &str) -> &'static str {
    match severity {
        "blocked" | "error" => "degraded",
        "warning" => "warning",
        _ => "ok",
    }
}

fn doctor_evidence_path(area: &str) -> &'static str {
    match area {
        "install" => "/checks/install/details",
        "paths" => "/checks/paths/details",
        "routing" => "/checks/routing/details",
        "plugins" => "/checks/plugins/details",
        "apps" => "/checks/apps/details",
        "shims" => "/checks/shims/details",
        "python" => "/checks/python/details",
        _ => "/checks",
    }
}

fn default_doctor_remediation(area: &str, message: &str) -> String {
    match area {
        "install" => {
            "run `bijux doctor` and align PATH/wheel installs so one canonical runtime is active"
                .to_string()
        }
        "paths" => "fix file permissions or configure BIJUX state path overrides".to_string(),
        "plugins" => {
            "run `bijux plugins doctor` and repair incompatible plugin entries".to_string()
        }
        "apps" => {
            "run `bijux apps doctor` and resolve mount metadata or runtime entrypoints".to_string()
        }
        "shims" => "remove deprecated alias binaries and keep declared product binaries on PATH"
            .to_string(),
        "python" => "install a supported Python runtime and validate `bijux_cli_py` import parity"
            .to_string(),
        "routing" => "inspect route inventory and clear namespace collisions".to_string(),
        _ => {
            format!("inspect doctor findings for `{area}` and apply the documented remediation: {message}")
        }
    }
}

fn doctor_issue_with_remediation(
    area: &str,
    severity: &str,
    message: impl Into<String>,
    remediation: Option<String>,
) -> Value {
    let message = message.into();
    json!({
        "area": area,
        "affected_surface": area,
        "severity": severity,
        "message": message,
        "evidence_path": doctor_evidence_path(area),
        "remediation": remediation.unwrap_or_else(|| default_doctor_remediation(area, &message)),
    })
}

fn doctor_issue(area: &str, severity: &str, message: impl Into<String>) -> Value {
    doctor_issue_with_remediation(area, severity, message, None)
}

fn doctor_check(
    name: &str,
    severity: &str,
    message: impl Into<String>,
    details: Value,
    suggestions: Vec<String>,
) -> Value {
    json!({
        "name": name,
        "severity": severity,
        "status": status_from_severity(severity),
        "message": message.into(),
        "details": details,
        "suggestions": suggestions,
    })
}

fn aggregate_suggestions(checks: &[Value]) -> Vec<Value> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for check in checks {
        let Some(items) = check.get("suggestions").and_then(Value::as_array) else {
            continue;
        };
        for item in items.iter().filter_map(Value::as_str) {
            if seen.insert(item.to_string()) {
                out.push(json!(item));
            }
        }
    }
    out
}

fn issue_status(issues: &[Value]) -> &'static str {
    if issues.iter().any(|item| {
        item.get("status") == Some(&json!("error"))
            || item.get("severity") == Some(&json!("error"))
            || item.get("severity") == Some(&json!("blocked"))
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

fn install_doctor_check(install: &Value) -> (Value, Vec<Value>) {
    let mut issues = Vec::<Value>::new();
    let mut suggestions = Vec::<String>::new();

    if install["has_path_shadowing"] == json!(true) {
        issues.push(doctor_issue(
            "install",
            "warning",
            "multiple bijux binaries are visible on PATH",
        ));
        suggestions.push(
            "Remove older bijux binaries from PATH so one canonical runtime resolves first."
                .to_string(),
        );
    }
    if install["has_duplicate_installs"] == json!(true) {
        issues.push(doctor_issue("install", "warning", "duplicate bijux installs were detected"));
        suggestions.push(
            "Keep either the cargo or Python install path active and remove the duplicate install."
                .to_string(),
        );
    }
    if install["has_mismatched_wheel_binary_versions"] == json!(true) {
        issues.push(doctor_issue("install", "warning", "wheel and binary versions do not match"));
        suggestions.push(
            "Reinstall bijux so the Python wheel and active binary come from the same release."
                .to_string(),
        );
    }
    if install["stale_wrapper_scripts"].as_array().is_some_and(|items| !items.is_empty()) {
        issues.push(doctor_issue("install", "warning", "stale wrapper scripts were found"));
        suggestions.push(
            "Remove stale wrapper scripts or replace them with a canonical `bijux` binary."
                .to_string(),
        );
    }
    if install["legacy_installer_conflicts"].as_array().is_some_and(|items| !items.is_empty()) {
        issues.push(doctor_issue("install", "warning", "legacy installer conflicts were found"));
        suggestions.push(
            "Remove legacy installer shims so only supported bijux entrypoints remain on PATH."
                .to_string(),
        );
    }

    let severity = max_severity(
        issues.iter().filter_map(|issue| issue.get("severity").and_then(Value::as_str)),
    );
    let check = doctor_check(
        "install",
        severity,
        if issues.is_empty() {
            "runtime install paths evaluated"
        } else {
            "install surface needs attention"
        },
        install.clone(),
        suggestions,
    );
    (check, issues)
}

fn state_paths_doctor_report(paths: &ResolvedStatePaths) -> Value {
    let query = state_diagnostics_query(
        &paths.config_file,
        &paths.history_file,
        &paths.plugin_registry_file,
        &paths.memory_file,
    );
    let diagnostics = state_diagnostics(paths);
    let mut issues =
        diagnostics.get("issues").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut suggestions = BTreeSet::new();

    let statuses = [
        ("config", &query.config),
        ("history", &query.history),
        ("plugin_registry", &query.plugins_registry),
        ("memory", &query.memory),
    ];
    for (label, status) in statuses {
        if status.exists && !status.readable {
            issues.push(doctor_issue(
                "paths",
                "error",
                format!("{label} is not readable: {}", status.path.display()),
            ));
            suggestions.insert(format!(
                "Grant read access for {} or move it to a readable location.",
                status.path.display()
            ));
        }
        if !status.writable {
            issues.push(doctor_issue(
                "paths",
                "warning",
                format!("{label} is not writable: {}", status.path.display()),
            ));
            suggestions.insert(format!(
                "Grant write access for {} or update BIJUX state path overrides.",
                status.path.display()
            ));
        }
    }

    let severity = max_severity(
        issues.iter().filter_map(|issue| issue.get("severity").and_then(Value::as_str)),
    );

    json!({
        "status": status_from_severity(severity),
        "severity": severity,
        "paths": {
            "config": crate::features::diagnostics::state_paths::state_path_status_value(&query.config),
            "history": crate::features::diagnostics::state_paths::state_path_status_value(&query.history),
            "plugin_registry": crate::features::diagnostics::state_paths::state_path_status_value(&query.plugins_registry),
            "memory": crate::features::diagnostics::state_paths::state_path_status_value(&query.memory),
        },
        "compatibility_config": paths.compatibility_config_file,
        "compatibility_config_warning": paths.compatibility_config_warning,
        "issues": issues,
        "suggestions": suggestions.into_iter().map(Value::from).collect::<Vec<_>>(),
        "diagnostics": diagnostics,
    })
}

fn routing_doctor_report(registry: &RouteRegistry) -> Value {
    let route_inventory = route_inventory(registry);
    let registry_inventory = registry_inventory(registry);
    json!({
        "status": "ok",
        "severity": "ok",
        "summary": {
            "route_count": route_inventory.routes.len(),
            "alias_count": route_inventory.aliases.len(),
            "namespace_count": registry_inventory.namespaces.len(),
        },
        "routes": route_inventory.routes,
        "aliases": route_inventory.aliases,
        "namespaces": registry_inventory.namespaces,
        "issues": [],
        "suggestions": [
            "Use `bijux inspect --format json` when you need runtime route-source provenance."
        ],
    })
}

fn route_inventory_export_report(registry: &RouteRegistry) -> Value {
    let routing = routing_doctor_report(registry);
    let shims = shim_doctor_report();
    json!({
        "status": "ok",
        "schema_version": "bijux-cli-route-inventory-v1",
        "inventory": {
            "builtins": routing["routes"],
            "aliases": routing["aliases"],
            "namespaces": routing["namespaces"],
            "legacy_app_shims": shims["legacy_app_shims"],
            "legacy_installer_conflicts": shims["legacy_installer_conflicts"],
        },
        "summary": {
            "route_count": routing["summary"]["route_count"],
            "alias_count": routing["summary"]["alias_count"],
            "namespace_count": routing["summary"]["namespace_count"],
            "legacy_shim_count": shims["legacy_app_shims"]
                .as_array()
                .map(|rows| rows.len())
                .unwrap_or(0),
            "legacy_installer_conflict_count": shims["legacy_installer_conflicts"]
                .as_array()
                .map(|rows| rows.len())
                .unwrap_or(0),
        }
    })
}

fn shim_doctor_report() -> Value {
    let path_value = env::var("PATH").unwrap_or_default();
    let mut legacy_app_shims = Vec::<Value>::new();
    for tool in crate::contracts::known_bijux_tools() {
        let mut shim_names =
            tool.aliases.iter().map(|alias| format!("bijux-{alias}")).collect::<Vec<_>>();
        shim_names.sort();
        shim_names.dedup();
        for shim_name in shim_names {
            let paths = discover_named_path_binaries(&path_value, &shim_name);
            if !paths.is_empty() {
                legacy_app_shims.push(json!({
                    "namespace": tool.namespace,
                    "shim": shim_name,
                    "paths": paths,
                }));
            }
        }
    }

    let legacy_installer_conflicts =
        crate::features::install::legacy_installer_conflicts(&path_value);
    let mut suggestions = Vec::<String>::new();
    let mut issues = Vec::<Value>::new();
    if !legacy_app_shims.is_empty() {
        issues.push(doctor_issue(
            "shims",
            "warning",
            "deprecated app alias binaries were found on PATH",
        ));
        suggestions.push(
            "Remove deprecated alias binaries such as `bijux-workflow`. Keep declared product binaries on PATH, or route through `bijux <app> ...`."
                .to_string(),
        );
    }
    if !legacy_installer_conflicts.is_empty() {
        issues.push(doctor_issue(
            "shims",
            "warning",
            "legacy installer wrappers were found on PATH",
        ));
        suggestions.push(
            "Remove legacy installer wrapper files so they cannot shadow supported bijux entrypoints."
                .to_string(),
        );
    }
    let severity = max_severity(
        issues.iter().filter_map(|issue| issue.get("severity").and_then(Value::as_str)),
    );
    let policy_status = if legacy_app_shims.is_empty() && legacy_installer_conflicts.is_empty() {
        "clear"
    } else if !legacy_installer_conflicts.is_empty() {
        "refused"
    } else {
        "deprecated"
    };
    json!({
        "status": status_from_severity(severity),
        "severity": severity,
        "lifecycle_policy": {
            "shim_support": "deprecated",
            "preferred_invocation": "declared product binaries or bijux <app> routes",
            "shadowing_policy": "refused",
            "policy_status": policy_status,
        },
        "legacy_app_shims": legacy_app_shims,
        "legacy_installer_conflicts": legacy_installer_conflicts,
        "issues": issues,
        "suggestions": suggestions,
    })
}

fn python_bridge_doctor_report() -> Value {
    let interpreters = detected_python_interpreters()
        .into_iter()
        .map(|candidate| {
            json!({
                "command": candidate.command,
                "version": candidate.version,
                "supported": candidate.supported,
            })
        })
        .collect::<Vec<_>>();

    let mut issues = Vec::<Value>::new();
    let mut suggestions = Vec::<String>::new();
    let configured_interpreter =
        env::var_os("BIJUX_PYTHON_BIN").map(|value| value.to_string_lossy().into_owned());
    let active_venv = env::var_os("VIRTUAL_ENV").map(|value| value.to_string_lossy().into_owned());

    let mut bridge = json!({
        "module": "bijux_cli_py",
        "module_spec": Value::Null,
        "import_ok": false,
        "import_error": Value::Null,
        "package_version": Value::Null,
        "console_scripts": Vec::<Value>::new(),
    });

    let selected = resolve_python_interpreter();
    if selected.is_none() {
        issues.push(doctor_issue(
            "python",
            "warning",
            "python 3.11 or newer was not discovered for the bridge runtime",
        ));
        suggestions.push(
            "Install Python 3.11 or newer, or point `BIJUX_PYTHON_BIN` at a supported interpreter."
                .to_string(),
        );
    }

    if let Some(interpreter) = &selected {
        match Command::new(&interpreter.command).arg("-c").arg(PYTHON_BRIDGE_DOCTOR_PROBE).output()
        {
            Ok(output) if output.status.success() => {
                if let Ok(payload) = serde_json::from_slice::<Value>(&output.stdout) {
                    bridge = payload;
                } else {
                    issues.push(doctor_issue(
                        "python",
                        "warning",
                        "python bridge probe returned invalid JSON",
                    ));
                }
            }
            Ok(output) => {
                issues.push(doctor_issue(
                    "python",
                    "warning",
                    format!(
                        "python bridge probe exited with {}",
                        output.status.code().unwrap_or(1)
                    ),
                ));
            }
            Err(error) => {
                issues.push(doctor_issue(
                    "python",
                    "warning",
                    format!("failed to launch python bridge probe: {error}"),
                ));
            }
        }
    }

    if bridge.get("import_ok") != Some(&json!(true)) {
        issues.push(doctor_issue(
            "python",
            "warning",
            "the active Python runtime cannot import `bijux_cli_py`",
        ));
        suggestions.push(
            "Install the `bijux-cli` Python package in the selected interpreter when Python bridge parity matters."
                .to_string(),
        );
    }

    if bridge.get("console_scripts").and_then(Value::as_array).is_none_or(|items| items.is_empty())
    {
        issues.push(doctor_issue(
            "python",
            "info",
            "no Python console-script entrypoints were discovered for `bijux-cli`",
        ));
        suggestions.push(
            "Use `python -m bijux_cli_py version` or install the wheel with console scripts when validating Python packaging."
                .to_string(),
        );
    }

    let severity = max_severity(
        issues.iter().filter_map(|issue| issue.get("severity").and_then(Value::as_str)),
    );

    json!({
        "status": status_from_severity(severity),
        "severity": severity,
        "environment": {
            "virtual_env": active_venv,
            "configured_interpreter": configured_interpreter,
        },
        "selected_interpreter": selected.as_ref().map(|candidate| json!({
            "command": candidate.command,
            "version": candidate.version,
            "supported": candidate.supported,
        })),
        "interpreters": interpreters,
        "bridge": bridge,
        "issues": issues,
        "suggestions": suggestions,
    })
}

fn doctor_bundle_root() -> Result<std::path::PathBuf, String> {
    let cwd = env::current_dir()
        .map_err(|error| format!("failed to resolve current directory: {error}"))?;
    Ok(cwd.join("artifacts").join("bijux-cli").join("doctor-bundle"))
}

fn write_bundle_text(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    crate::infrastructure::fs_store::atomic_write_text(path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn write_bundle_json(path: &Path, payload: &Value) -> Result<(), String> {
    let rendered = serde_json::to_string_pretty(payload)
        .map_err(|error| format!("failed to render JSON: {error}"))?;
    write_bundle_text(path, &(rendered + "\n"))
}

fn export_doctor_bundle(report: &Value) -> Result<Value, String> {
    let root = doctor_bundle_root()?;
    let docs_report = docs_inventory_report();
    let config_docs = schema_docs_report(None).map_err(|error| error.to_string())?;
    let config_markdown = config_docs
        .get("markdown")
        .and_then(Value::as_str)
        .ok_or_else(|| "config docs report is missing markdown".to_string())?;

    let doctor_path = root.join("doctor.json");
    let docs_path = root.join("docs.json");
    let config_path = root.join("generated-config-reference.md");
    let readme_path = root.join("README.txt");

    write_bundle_json(&doctor_path, report)?;
    write_bundle_json(&docs_path, &docs_report)?;
    write_bundle_text(&config_path, config_markdown)?;
    write_bundle_text(
        &readme_path,
        "bijux doctor bundle\n\nFiles:\n- doctor.json\n- docs.json\n- generated-config-reference.md\n",
    )?;

    Ok(json!({
        "status": "ok",
        "path": root,
        "files": [
            doctor_path,
            docs_path,
            config_path,
            readme_path,
        ],
    }))
}

pub(crate) fn doctor_report(
    paths: &ResolvedStatePaths,
    registry: &RouteRegistry,
    plugin_registry_path: &Path,
) -> Value {
    let install = install_report_payload();
    let (install_check, mut issues) = install_doctor_check(&install);
    let state = state_paths_doctor_report(paths);
    let shims = shim_doctor_report();
    let routing = routing_doctor_report(registry);
    let python = python_bridge_doctor_report();
    let apps = serde_json::to_value(apps_doctor_report(paths, plugin_registry_path))
        .expect("apps doctor report");
    let plugins = match plugin_doctor(plugin_registry_path) {
        Ok(report) => {
            let severity = if report.broken.is_empty() && report.incompatible.is_empty() {
                "ok"
            } else {
                "warning"
            };
            let suggestions = if severity == "ok" {
                Vec::new()
            } else {
                vec![
                    "Run `bijux plugins doctor` or `bijux plugins inspect <namespace>` to repair incompatible plugins."
                        .to_string(),
                ]
            };
            doctor_check(
                "plugins",
                severity,
                if severity == "ok" {
                    "plugin registry health evaluated"
                } else {
                    "installed plugins need attention"
                },
                json!({
                    "installed": report.installed,
                    "broken": report.broken,
                    "incompatible": report.incompatible,
                }),
                suggestions,
            )
        }
        Err(error) => doctor_check(
            "plugins",
            "error",
            error.to_string(),
            json!({ "status": "unavailable" }),
            vec!["Repair or recreate the plugin registry file before relying on plugin routes."
                .to_string()],
        ),
    };
    let apps_check = doctor_check(
        "apps",
        if apps["status"] == json!("ok") { "ok" } else { "warning" },
        if apps["status"] == json!("ok") {
            "official app discovery is healthy"
        } else {
            "some official app mounts need attention"
        },
        apps.clone(),
        vec!["Run `bijux doctor <app>` for one app or `bijux apps which <app>` for the resolved entrypoint."
            .to_string()],
    );
    let routing_check = doctor_check(
        "routing",
        "ok",
        "route registry inventory evaluated",
        routing.clone(),
        vec!["Run `bijux doctor routing --format json` for the full route and alias inventory."
            .to_string()],
    );
    let state_check = doctor_check(
        "paths",
        state["severity"].as_str().unwrap_or("ok"),
        if state["issues"].as_array().is_some_and(|items| items.is_empty()) {
            "runtime state files evaluated"
        } else {
            "runtime state paths need attention"
        },
        state.clone(),
        state["suggestions"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect(),
    );
    let shims_check = doctor_check(
        "shims",
        shims["severity"].as_str().unwrap_or("ok"),
        if shims["issues"].as_array().is_some_and(|items| items.is_empty()) {
            "legacy shim scan completed"
        } else {
            "legacy compatibility shims were found"
        },
        shims.clone(),
        shims["suggestions"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect(),
    );
    let python_check = doctor_check(
        "python",
        python["severity"].as_str().unwrap_or("ok"),
        if python["issues"].as_array().is_some_and(|items| items.is_empty()) {
            "python bridge runtime evaluated"
        } else {
            "python bridge runtime needs attention"
        },
        python.clone(),
        python["suggestions"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect(),
    );

    let checks = vec![
        install_check,
        state_check,
        plugins,
        apps_check,
        routing_check,
        shims_check,
        python_check,
    ];
    for report in [&state, &shims, &python] {
        if let Some(items) = report.get("issues").and_then(Value::as_array) {
            issues.extend(items.iter().cloned());
        }
    }
    for check in &checks {
        let severity = check.get("severity").and_then(Value::as_str).unwrap_or("ok");
        if severity != "ok" {
            let area = check["name"].as_str().unwrap_or("unknown");
            let message =
                check["message"].as_str().unwrap_or("doctor check reported a non-ok state");
            let remediation = check
                .get("suggestions")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            issues.push(doctor_issue_with_remediation(area, severity, message, remediation));
        }
    }
    let severity = max_severity(
        checks.iter().filter_map(|check| check.get("severity").and_then(Value::as_str)),
    );

    json!({
        "status": status_from_severity(severity),
        "severity": severity,
        "checks": checks,
        "install": {
            "has_path_shadowing": install["has_path_shadowing"],
            "has_duplicate_installs": install["has_duplicate_installs"],
            "stale_wrapper_scripts": install["stale_wrapper_scripts"],
            "legacy_installer_conflicts": install["legacy_installer_conflicts"].as_array().is_some_and(|items| !items.is_empty()),
            "legacy_installer_conflict_paths": install["legacy_installer_conflicts"],
            "has_mismatched_wheel_binary_versions": install["has_mismatched_wheel_binary_versions"],
        },
        "routing": routing,
        "paths": state,
        "apps": apps,
        "shims": shims,
        "python": python,
        "issues": issues,
        "suggestions": aggregate_suggestions(&checks),
    })
}

fn doctor_topic_report(
    argv: &[String],
    paths: &ResolvedStatePaths,
    registry: &RouteRegistry,
    plugin_registry_path: &Path,
) -> Value {
    let subject = command_positionals(argv, &["cli", "doctor"]).first().cloned();
    match subject.as_deref() {
        None => {
            let report = doctor_report(paths, registry, plugin_registry_path);
            if command_has_flag(argv, "--bundle") {
                match export_doctor_bundle(&report) {
                    Ok(bundle) => {
                        let mut object = report.as_object().cloned().unwrap_or_default();
                        object.insert("bundle".to_string(), bundle);
                        Value::Object(object)
                    }
                    Err(error) => runtime_error_payload(error),
                }
            } else {
                report
            }
        }
        Some("routing") => routing_doctor_report(registry),
        Some("paths") => state_paths_doctor_report(paths),
        Some("apps") => serde_json::to_value(apps_doctor_report(paths, plugin_registry_path))
            .expect("apps doctor report"),
        Some("shims") => shim_doctor_report(),
        Some("python") => python_bridge_doctor_report(),
        Some(query) => match app_doctor_report(query, paths) {
            Ok(report) => serde_json::to_value(report).expect("app doctor report"),
            Err(_) => runtime_error_payload(format!("unknown doctor topic: {query}")),
        },
    }
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
        ("root-cli-architecture", "docs/bijux-cli/architecture/root-cli-architecture.md"),
        ("app-integration", "docs/bijux-cli/interfaces/app-integration-guide.md"),
        ("generated-config-reference", "docs/bijux-cli/interfaces/generated-config-reference.md"),
        ("config-guide", "docs/bijux-cli/interfaces/config-guide.md"),
        ("installation", "docs/bijux-cli/operations/installation-and-setup.md"),
        ("migration-guide", "docs/bijux-cli/operations/migration-guide.md"),
        ("diagnostics-guide", "docs/bijux-cli/operations/diagnostics-guide.md"),
        ("plugin-workflows", "docs/bijux-cli/interfaces/operator-workflows.md"),
        ("python-distribution", "docs/bijux-cli/packages/bijux-cli-python.md"),
        ("examples", "docs/bijux-cli/interfaces/examples.md"),
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

fn script_contract_report() -> Value {
    json!({
        "status": "ok",
        "schema_version": "bijux-cli-script-contract-v1",
        "stable_for_automation": {
            "status": ["status", "runtime", "state", "plugins", "issues"],
            "doctor": ["status", "severity", "checks", "issues", "suggestions"],
            "version": ["name", "version", "semver", "source", "git_commit", "git_dirty", "build_profile"],
            "config_explain": ["status", "key", "logical_key", "storage_key", "effective", "layers", "environment"],
            "config_diff": ["status", "key", "from_profile", "to_profile", "changed_count", "changes"],
            "explain": ["status", "requested_command", "requested_path", "normalized_path", "route", "envelope"],
        },
        "unstable_human_text": [
            "Rendered `text` output intended for human operators",
            "Help prose, examples, and suggestion wording",
            "Diagnostics message phrasing outside machine fields",
        ],
        "safe_output_modes": ["json", "jsonl", "yaml"],
        "unsafe_for_parsing": ["text"],
        "exit_code_contract": {
            "0": "success",
            "1": "runtime_error",
            "2": "usage_or_validation_error",
            "3": "encoding_error",
            "130": "aborted",
        }
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
        [a, b] if a == "cli" && b == "doctor" => {
            Some(doctor_topic_report(argv, paths, registry, plugin_registry_path))
        }
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
                    "schemas": [
                        "command-envelope-v1",
                        "output-envelope-v1",
                        "error-envelope-v1",
                        "config-schema-registry-v1",
                        "plugin-manifest-v2",
                        "product-mount-descriptor-v1"
                    ],
                    "version": "v1",
                }
            }))
        }
        [a, b] if a == "cli" && b == "status" => {
            Some(runtime_status_report(paths, plugin_registry_path))
        }
        [a, b] if a == "cli" && b == "routes" => Some(route_inventory_export_report(registry)),
        [a, b] if a == "cli" && b == "shims" => Some(shim_doctor_report()),
        [a, b] if a == "cli" && b == "script-contract" => Some(script_contract_report()),
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
    use std::path::Path;

    use tempfile::tempdir;

    use super::{
        completion_report, docs_inventory_report_at, doctor_report, doctor_topic_report,
        runtime_audit_report, runtime_status_report,
    };
    use crate::features::diagnostics::state_paths::ResolvedStatePaths;
    use crate::routing::registry::RouteRegistry;
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
        let report = doctor_report(&paths, &RouteRegistry::default(), &paths.plugin_registry_file);

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
        assert_eq!(report["severity"], serde_json::json!("warning"));
        assert!(report["suggestions"].as_array().is_some_and(|items| !items.is_empty()));
        assert!(report["issues"].as_array().expect("issues").iter().all(|issue| issue
            .get("affected_surface")
            .is_some()
            && issue.get("evidence_path").is_some()
            && issue.get("remediation").is_some()));
    }

    #[test]
    fn doctor_paths_topic_reports_permission_surface() {
        let temp = tempdir().expect("temp dir");
        let paths = ResolvedStatePaths {
            config_file: temp.path().join("config.env"),
            history_file: temp.path().join("history.txt"),
            plugins_dir: temp.path().join("plugins"),
            plugin_registry_file: temp.path().join("plugins/registry.json"),
            memory_file: temp.path().join("memory.json"),
            compatibility_config_file: temp.path().join("compatibility.env"),
            compatibility_config_warning: Some("compat warning".to_string()),
        };
        let argv = vec!["bijux".to_string(), "doctor".to_string(), "paths".to_string()];

        let report = doctor_topic_report(
            &argv,
            &paths,
            &RouteRegistry::default(),
            &paths.plugin_registry_file,
        );

        assert!(report["paths"]["config"]["path"].is_string());
        assert!(report["suggestions"].as_array().is_some());
        assert_eq!(report["compatibility_config_warning"], serde_json::json!("compat warning"));
    }

    #[test]
    fn doctor_routing_topic_reports_registry_inventory() {
        let temp = tempdir().expect("temp dir");
        let paths = ResolvedStatePaths {
            config_file: temp.path().join("config.env"),
            history_file: temp.path().join("history.txt"),
            plugins_dir: temp.path().join("plugins"),
            plugin_registry_file: temp.path().join("plugins/registry.json"),
            memory_file: temp.path().join("memory.json"),
            compatibility_config_file: temp.path().join("compatibility.env"),
            compatibility_config_warning: None,
        };
        let argv = vec!["bijux".to_string(), "doctor".to_string(), "routing".to_string()];

        let report = doctor_topic_report(
            &argv,
            &paths,
            &RouteRegistry::default(),
            &paths.plugin_registry_file,
        );

        assert_eq!(report["status"], serde_json::json!("ok"));
        assert!(report["summary"]["route_count"].as_u64().unwrap_or_default() > 0);
        assert!(report["aliases"].as_array().is_some_and(|items| !items.is_empty()));
    }

    #[test]
    fn doctor_python_topic_reports_bridge_inventory() {
        let temp = tempdir().expect("temp dir");
        let paths = ResolvedStatePaths {
            config_file: temp.path().join("config.env"),
            history_file: temp.path().join("history.txt"),
            plugins_dir: temp.path().join("plugins"),
            plugin_registry_file: temp.path().join("plugins/registry.json"),
            memory_file: temp.path().join("memory.json"),
            compatibility_config_file: temp.path().join("compatibility.env"),
            compatibility_config_warning: None,
        };
        let argv = vec!["bijux".to_string(), "doctor".to_string(), "python".to_string()];

        let report = doctor_topic_report(
            &argv,
            &paths,
            &RouteRegistry::default(),
            &paths.plugin_registry_file,
        );

        assert!(report["interpreters"].is_array());
        assert!(report["bridge"].is_object());
        assert!(report["environment"].is_object());
    }

    #[test]
    fn docs_inventory_references_generated_guides() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = docs_inventory_report_at(&workspace_root);
        let references = report["references"].as_array().expect("references");
        assert!(references
            .iter()
            .any(|item| item["path"] == "docs/bijux-cli/interfaces/generated-config-reference.md"));
        assert!(references
            .iter()
            .any(|item| item["path"] == "docs/bijux-cli/operations/migration-guide.md"));
    }

    #[test]
    fn doctor_unknown_topic_returns_runtime_error_payload() {
        let temp = tempdir().expect("temp dir");
        let paths = ResolvedStatePaths {
            config_file: temp.path().join("config.env"),
            history_file: temp.path().join("history.txt"),
            plugins_dir: temp.path().join("plugins"),
            plugin_registry_file: temp.path().join("plugins/registry.json"),
            memory_file: temp.path().join("memory.json"),
            compatibility_config_file: temp.path().join("compatibility.env"),
            compatibility_config_warning: None,
        };
        let argv = vec!["bijux".to_string(), "doctor".to_string(), "unknown-topic".to_string()];

        let report = doctor_topic_report(
            &argv,
            &paths,
            &RouteRegistry::default(),
            &paths.plugin_registry_file,
        );

        assert_eq!(report["status"], serde_json::json!("error"));
        assert_eq!(report["message"], serde_json::json!("unknown doctor topic: unknown-topic"));
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
